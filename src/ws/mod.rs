mod connect;
mod connection_details;

use connect::ws_upgrade;
use connection_details::ConnectionDetails;
use futures_util::{
    lock::Mutex,
    stream::{SplitSink, SplitStream},
    StreamExt, TryStreamExt,
};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_tungstenite::{self, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

use crate::{app_env::AppEnv, app_error::AppError, ws::ws_sender::WSSender, C};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WSReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type WSWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

mod ws_sender;

const AUTO_CLOSE_TIME: Duration = std::time::Duration::from_secs(40);

#[derive(Debug, Default)]
struct AutoClose(Option<JoinHandle<()>>);

/// Will close the connection after 40 seconds unless a ping message is received
impl AutoClose {
    fn init(&mut self, ws_sender: &WSSender) {
        if let Some(handle) = self.0.as_ref() {
            handle.abort();
        };
        let ws_sender = C!(ws_sender);
        self.0 = Some(tokio::spawn(async move {
            tokio::time::sleep(AUTO_CLOSE_TIME).await;
            ws_sender.close().await;
        }));
    }
}

/// Handle each incoming ws message
async fn incoming_ws_message(mut reader: WSReader, ws_sender: WSSender) {
    let mut auto_close = AutoClose::default();
    auto_close.init(&ws_sender);
    while let Ok(Some(message)) = reader.try_next().await {
        match message {
            Message::Text(message) => {
                let mut ws_sender = C!(ws_sender);
                tokio::spawn(async move {
                    ws_sender.on_text(message).await;
                });
            }
            Message::Ping(_) => auto_close.init(&ws_sender),
            Message::Close(_) => {
                ws_sender.close().await;
                break;
            }
            _ => (),
        };
    }
    info!("incoming_ws_message done");
}

/// need to spawn a new receiver on each connect
/// try to open WS connection, and spawn a ThreadChannel message handler
pub async fn open_connection(app_envs: AppEnv) -> Result<(), AppError> {
    let mut connection_details = ConnectionDetails::new();
    loop {
        info!("in connection loop, awaiting delay then try to connect");
        connection_details.reconnect_delay().await;

        match ws_upgrade(&app_envs).await {
            Ok(socket) => {
                info!("connected in ws_upgrade match");
                connection_details.valid_connect();

                let (writer, reader) = socket.split();

                let ws_sender = WSSender::new(
                    &app_envs,
                    connection_details.get_connect_instant(),
                    Arc::new(Mutex::new(writer)),
                );
                ws_sender.send_status().await;
                incoming_ws_message(reader, ws_sender).await;

                info!("aborted spawns, incoming_ws_message done, reconnect next");
            }
            Err(e) => {
                error!("connection::{e}");
                connection_details.fail_connect();
            }
        }
    }
}

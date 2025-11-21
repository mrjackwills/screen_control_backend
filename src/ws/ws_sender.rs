use async_channel::Sender;
use std::{process, time::Instant};

use crate::C;
use crate::message_handler::Msg;
use crate::sysinfo::SysInfo;
use crate::ws_messages::{MessageValues, ParsedMessage, PiStatus, Response};
use crate::{app_env::AppEnv, ws_messages::to_struct};

#[derive(Debug, Clone)]
pub struct WSSender {
    app_envs: AppEnv,
    connected_instant: Instant,
    tx: Sender<Msg>,
}

impl WSSender {
    pub fn new(app_envs: &AppEnv, tx: &Sender<Msg>) -> Self {
        Self {
            app_envs: C!(app_envs),
            connected_instant: std::time::Instant::now(),
            tx: C!(tx),
        }
    }

    /// Update the connected_instance time
    pub fn on_connection(&mut self) {
        self.connected_instant = std::time::Instant::now();
    }

    /// Handle text message, in this program they will all be json text
    pub async fn on_text(&self, message: String) {
        if let Some(data) = to_struct(&message) {
            match data {
                MessageValues::Invalid(error) => tracing::error!("invalid::{error:?}"),
                MessageValues::Valid(message) => match message {
                    ParsedMessage::ScreenOff => {
                        self.tx.send(Msg::ScreenOff).await.ok();
                    }
                    ParsedMessage::Status => {
                        self.tx.send(Msg::Status).await.ok();
                    }
                    ParsedMessage::ScreenOn => {
                        self.tx.send(Msg::ScreenOn).await.ok();
                    }
                },
            }
        }
    }

    async fn send_ws_response(&self, response: Response) {
        match self.tx.send(Msg::ToSend(response)).await {
            Ok(()) => (),
            Err(e) => {
                tracing::error!("{e}");
                process::exit(1);
            }
        }
    }

    /// Generate, and send, pi information
    pub async fn send_status(&self) {
        let sys_info = SysInfo::new(&self.app_envs).await;
        let pi_info = PiStatus::new(sys_info, self.connected_instant.elapsed().as_secs());
        self.send_ws_response(Response::Status(pi_info)).await;
    }
}

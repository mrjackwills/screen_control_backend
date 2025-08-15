use futures_util::SinkExt;
use futures_util::lock::Mutex;
use std::{process, sync::Arc, time::Instant};

use crate::sysinfo::SysInfo;
use crate::ws_messages::{MessageValues, ParsedMessage, PiStatus, Response, StructuredResponse};
use crate::{C, S, sleep};
use crate::{app_env::AppEnv, ws_messages::to_struct};

use super::WSWriter;

#[derive(Debug, Clone)]
pub struct WSSender {
    app_envs: AppEnv,
    connected_instant: Instant,
    writer: Arc<Mutex<WSWriter>>,
    unique: Option<String>,
}

impl WSSender {
    pub fn new(
        app_envs: &AppEnv,
        connected_instant: Instant,
        writer: Arc<Mutex<WSWriter>>,
    ) -> Self {
        Self {
            app_envs: C!(app_envs),
            connected_instant,
            writer,
            unique: None,
        }
    }

    /// Handle text message, in this program they will all be json text
    pub async fn on_text(&mut self, message: String) {
        if let Some(data) = to_struct(&message) {
            match data {
                MessageValues::Invalid(error) => tracing::error!("invalid::{error:?}"),
                MessageValues::Valid(msg, unique) => {
                    self.unique = Some(unique);
                    match msg {
                        ParsedMessage::Status => (),
                        ParsedMessage::ScreenOff => {
                            if let Err(e) = SysInfo::turn_off().await {
                                tracing::error!("{e}");
                                self.send_error("Unable to turn OFF screen").await;
                            }
                        }
                        ParsedMessage::ScreenOn => {
                            if let Err(e) = SysInfo::turn_on().await {
                                tracing::error!("{e}");
                                self.send_error("Unable to turn ON screen").await;
                            }
                            // sleep here so that the screen status can get updated
                            sleep!(250);
                        }
                    }
                    self.send_status().await;
                }
            }
        }
    }

    /// Send a message to the socket
    async fn send_ws_response(&self, response: Response, unique: Option<String>) {
        match self
            .writer
            .lock()
            .await
            .send(StructuredResponse::data(response, unique))
            .await
        {
            Ok(()) => tracing::trace!("Message sent"),
            Err(e) => {
                tracing::error!("send_ws_response::SEND-ERROR::{e:?}");
                process::exit(1);
            }
        }
    }

    /// Send a unique error message
    pub async fn send_error(&self, message: &str) {
        self.send_ws_response(Response::Error(S!(message)), C!(self.unique))
            .await;
    }

    /// Generate, and send, pi information
    pub async fn send_status(&self) {
        let info = SysInfo::new(&self.app_envs).await;
        let info = PiStatus::new(info, self.connected_instant.elapsed().as_secs());
        self.send_ws_response(Response::Status(info), None).await;
    }

    /// close connection, uses a 2 second timeout
    pub async fn close(&self) {
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            self.writer.lock().await.close(),
        )
        .await
        .ok()
        .map(std::result::Result::ok);
    }
}

use jiff::Zoned;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::sysinfo::SysInfo;

use super::ScreenStatus;

/// Basic pi info
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PiStatus {
    pub ip_address: String,
    pub screen_status: Option<ScreenStatus>,
    pub time_off: (i8, i8),
    pub time_on: (i8, i8),
    pub timezone: String,
    pub uptime_app: u64,
    pub uptime_ws: u64,
    pub uptime: usize,
    pub version: String,
}
/// Combined pi into and current set alarms
impl PiStatus {
    pub fn new(sysinfo: SysInfo, uptime_ws: u64) -> Self {
        let zone = Zoned::now();
        Self {
            ip_address: sysinfo.ip_address,
            screen_status: sysinfo.screen_status,
            time_off: sysinfo.time_off,
            time_on: sysinfo.time_on,
            timezone: zone.time_zone().iana_name().unwrap_or("Etc/UTC").to_owned(),
            uptime_app: sysinfo.uptime_app,
            uptime: sysinfo.uptime,
            uptime_ws,
            version: sysinfo.version,
        }
    }
}
/// Responses, either sent as is, or nested in StructuredResponse below
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "name", content = "data")]
pub enum Response {
    Status(PiStatus),
    Error(String),
}

/// These get sent to the websocket server when in structured_data mode,
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct StructuredResponse {
    data: Option<Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Response>,
}

impl StructuredResponse {
    /// Convert a ResponseMessage into a Tokio message of StructureResponse
    pub fn data(data: Response) -> Message {
        let x = Self {
            data: Some(data),
            error: None,
        };
        Message::Text(serde_json::to_string(&x).unwrap_or_default().into())
    }

    /// Convert a ErrorResponse into a Tokio message of StructureResponse
    pub fn _error(data: Response) -> Message {
        let x = Self {
            error: Some(data),
            data: None,
        };
        Message::Text(serde_json::to_string(&x).unwrap_or_default().into())
    }
}

use std::process::Output;

use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

use crate::{S, app_env::AppEnv, app_error::AppError, ws_messages::ScreenStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct SysInfo {
    pub ip_address: String,
    pub screen_status: Option<ScreenStatus>,
    pub time_off: (i8, i8),
    pub time_on: (i8, i8),
    pub uptime_app: u64,
    pub uptime: usize,
    pub version: String,
}

impl SysInfo {
    /// Check the screen status, maybe put this value in an .env, as it can change depending which por
    pub async fn screen_status() -> Option<ScreenStatus> {
        let get = |num: u8| async move {
            read_to_string(format!("/sys/class/drm/card1-HDMI-A-{num}/enabled"))
                .await
                .unwrap_or_default()
                .trim()
                .to_owned()
        };

        let status = [get(1).await, get(2).await];

        if status.contains(&"enabled".into()) {
            Some(ScreenStatus::On)
        } else if status.contains(&"disabled".into()) {
            Some(ScreenStatus::Off)
        } else {
            None
        }
    }

    /// Attempt to toggle the status of the screen
    async fn toggle_screen(status: ScreenStatus) -> Result<Output, AppError> {
        let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
        let dbus_address = format!("unix:path=/run/user/{uid}/bus");
        tokio::process::Command::new("busctl")
            .args([
                "--user",
                "set-property",
                "org.gnome.Mutter.DisplayConfig",
                "/org/gnome/Mutter/DisplayConfig",
                "org.gnome.Mutter.DisplayConfig",
                "PowerSaveMode",
                "i",
                status.get_arg_value(),
            ])
            .env("DBUS_SESSION_BUS_ADDRESS", dbus_address)
            .output()
            .await
            .map_err(AppError::Io)
    }
    pub async fn turn_on() -> Result<Output, AppError> {
        Self::toggle_screen(ScreenStatus::On).await
    }
    pub async fn turn_off() -> Result<Output, AppError> {
        Self::toggle_screen(ScreenStatus::Off).await
    }

    /// Get uptime by reading, and parsing, /proc/uptime file
    async fn get_uptime() -> usize {
        let uptime = read_to_string("/proc/uptime").await.unwrap_or_default();
        let (uptime, _) = uptime.split_once('.').unwrap_or_default();
        uptime.parse::<usize>().unwrap_or_default()
    }

    /// Generate sysinfo struct, will valid data
    pub async fn new(app_envs: &AppEnv) -> Self {
        Self {
            ip_address: local_ip().map_or_else(|_| S!("UNKNOWN"), |i| i.to_string()),
            uptime: Self::get_uptime().await,
            uptime_app: std::time::SystemTime::now()
                .duration_since(app_envs.start_time)
                .map_or(0, |value| value.as_secs()),
            screen_status: Self::screen_status().await,
            time_on: (app_envs.time_on.hour(), app_envs.time_on.minute()),
            time_off: (app_envs.time_off.hour(), app_envs.time_off.minute()),
            version: S!(env!("CARGO_PKG_VERSION")),
        }
    }
}

// SysInfo tests
//
/// cargo watch -q -c -w src/ -x 'test sysinfo -- --test-threads=1 --nocapture'
#[cfg(test)]
mod tests {
    use crate::{sleep, tests::test_setup};

    use super::*;

    #[tokio::test]
    async fn sysinfo_getuptime_ok() {
        let result = SysInfo::get_uptime().await;

        // Assumes ones computer has been turned on for one minute
        assert!(result > 60);
    }

    #[tokio::test]
    async fn sysinfo_get_sysinfo_ok() {
        let app_envs = test_setup();
        sleep!(1000);

        let result = SysInfo::new(&app_envs).await;

        assert_eq!(result.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(result.uptime_app, 1);
        // Again assume ones computer has been turned on for one minute
        assert!(result.uptime > 60);
    }
}

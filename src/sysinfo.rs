use std::process::Output;

use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
// use tracing::error;

use crate::{app_env::AppEnv, app_error::AppError, ws_messages::ScreenStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct SysInfo {
    pub ip_address: String,
    pub uptime: usize,
    pub version: String,
    pub uptime_app: u64,
    pub screen_status: Option<ScreenStatus>,
}

const WLR: &str = "/usr/bin/wlr-randr";
const WLR_ARGS: [&str; 2] = ["--output", "HDMI-A-2"];
const ON: &str = "--on";
const OFF: &str = "--off";
const XDG_KEY: &str = "XDG_RUNTIME_DIR";
const XDG_VAL: &str = "/run/user/1000";
const WAY_KEY: &str = "WAYLAND_DISPLAY";
const WAY_VAL: &str = "wayland-1";

impl SysInfo {
    /// Set ENV's needed for waylad screen status/control
    pub fn set_wayland_env() {
        std::env::set_var(XDG_KEY, XDG_VAL);
        std::env::set_var(WAY_KEY, WAY_VAL);
    }

    /// (attempt) to turn on the screen
    pub async fn turn_on() -> Result<Output, AppError> {
        tokio::process::Command::new(WLR)
            .args(WLR_ARGS)
            .arg(ON)
            .output()
            .await
            .map_err(AppError::Io)
    }

    /// (attempt) to turn off the screen
    pub async fn turn_off() -> Result<Output, AppError> {
        tokio::process::Command::new(WLR)
            .args(WLR_ARGS)
            .arg(OFF)
            .output()
            .await
            .map_err(AppError::Io)
    }

    /// Convert from wlr-randr to an Option<ScreenStatus>
    fn extract_enabled_line(input: &str) -> Option<ScreenStatus> {
        input
            .lines()
            .find(|line| line.trim().starts_with("Enabled:"))
            .map(|x| {
                if x.trim().ends_with("no") {
                    ScreenStatus::Off
                } else {
                    ScreenStatus::On
                }
            })
    }

    /// Get the screen status
    pub async fn screen_status() -> Option<ScreenStatus> {
        Self::extract_enabled_line(
            &tokio::process::Command::new(WLR)
                .args(WLR_ARGS)
                .output()
                .await
                .map_or(String::new(), |i| {
                    String::from_utf8(i.stdout).unwrap_or_default()
                }),
        )
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
            ip_address: local_ip().map_or("UNKNOWN".to_owned(), |i| i.to_string()),
            uptime: Self::get_uptime().await,
            uptime_app: std::time::SystemTime::now()
                .duration_since(app_envs.start_time)
                .map_or(0, |value| value.as_secs()),
            screen_status: Self::screen_status().await,
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

// SysInfo tests
//
/// cargo watch -q -c -w src/ -x 'test sysinfo -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    #[tokio::test]
    async fn test_extract_enabled_line() {
        let off = r#"Command output: HDMI-A-2 "NewTek JOYSX 0000000000001 (HDMI-A-2)"
        Physical size: 480x270 mm
        Enabled: no
        Modes:
          720x400 px, 70.082001 Hz
          640x480 px, 59.939999 Hz
          640x480 px, 59.939999 Hz
          640x480 px, 60.000000 Hz
          640x480 px, 66.667000 Hz
          640x480 px, 72.808998 Hz
          640x480 px, 75.000000 Hz
          720x480 px, 59.939999 Hz
          720x480 px, 60.000000 Hz
          720x576 px, 50.000000 Hz
          800x600 px, 56.250000 Hz
          800x600 px, 60.317001 Hz
          800x600 px, 72.188004 Hz
          800x600 px, 75.000000 Hz
          832x624 px, 74.551003 Hz
          1024x768 px, 60.004002 Hz
          1024x768 px, 70.069000 Hz
          1024x768 px, 75.028999 Hz
          1280x720 px, 50.000000 Hz
          1280x720 px, 59.939999 Hz
          1280x720 px, 60.000000 Hz
          1280x720 px, 60.000000 Hz
          1366x768 px, 60.009998 Hz
          1280x960 px, 60.000000 Hz
          1440x900 px, 59.901001 Hz
          1280x1024 px, 60.020000 Hz
          1280x1024 px, 75.025002 Hz
          1680x1050 px, 59.882999 Hz
          1920x1080 px, 60.000000 Hz
          1024x600 px, 59.993000 Hz (preferred)"#;

        let result = SysInfo::extract_enabled_line(off);
        assert!(result.is_some());
        assert_eq!(result, Some(ScreenStatus::Off));

        let on = r#"Command output: HDMI-A-2 "NewTek JOYSX 0000000000001 (HDMI-A-2)"
        Physical size: 480x270 mm
        Enabled: yes
        Modes:
          720x400 px, 70.082001 Hz
          640x480 px, 59.939999 Hz
          640x480 px, 59.939999 Hz
          640x480 px, 60.000000 Hz
          640x480 px, 66.667000 Hz
          640x480 px, 72.808998 Hz
          640x480 px, 75.000000 Hz
          720x480 px, 59.939999 Hz
          720x480 px, 60.000000 Hz
          720x576 px, 50.000000 Hz
          800x600 px, 56.250000 Hz
          800x600 px, 60.317001 Hz
          800x600 px, 72.188004 Hz
          800x600 px, 75.000000 Hz
          832x624 px, 74.551003 Hz
          1024x768 px, 60.004002 Hz
          1024x768 px, 70.069000 Hz
          1024x768 px, 75.028999 Hz
          1280x720 px, 50.000000 Hz
          1280x720 px, 59.939999 Hz
          1280x720 px, 60.000000 Hz
          1280x720 px, 60.000000 Hz
          1366x768 px, 60.009998 Hz
          1280x960 px, 60.000000 Hz
          1440x900 px, 59.901001 Hz
          1280x1024 px, 60.020000 Hz
          1280x1024 px, 75.025002 Hz
          1680x1050 px, 59.882999 Hz
          1920x1080 px, 60.000000 Hz
          1024x600 px, 59.993000 Hz (preferred)"#;

        let result = SysInfo::extract_enabled_line(on);
        assert!(result.is_some());
        assert_eq!(result, Some(ScreenStatus::On));

        let result = SysInfo::extract_enabled_line("error with some random text");
        assert!(result.is_none());
    }
}

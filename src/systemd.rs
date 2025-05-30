use crate::S;
use crate::{CliArg, app_error::AppError};
use std::{env, fs, io::Write, path::Path, process::Command};

const SYSTEMCTL: &str = "systemctl";
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn check_sudo() -> Result<(), AppError> {
    match sudo::check() {
        sudo::RunningAs::Root => Ok(()),
        _ => Err(AppError::Internal(S!("not running as sudo"))),
    }
}

/// Get user name, to check if is sudo
fn get_user_name() -> Option<String> {
    std::env::var("SUDO_USER").map_or(None, |user_name| {
        if user_name == "root" || user_name.is_empty() {
            None
        } else {
            Some(user_name)
        }
    })
}

/// Check if unit file in systemd, and delete if true
fn uninstall_service() -> Result<(), AppError> {
    let service = get_service_name();

    let path = get_dot_service();

    if Path::new(&path).exists() {
        tracing::info!("Stopping service");
        Command::new(SYSTEMCTL).args(["stop", &service]).output()?;

        tracing::info!("Disabling service");
        Command::new(SYSTEMCTL)
            .args(["disable", &service])
            .output()?;

        tracing::info!("Removing service file");
        fs::remove_file(path)?;

        tracing::info!("Reload daemon-service");
        Command::new(SYSTEMCTL).arg("daemon-reload").output()?;
    }
    Ok(())
}

/// Get service name for systemd service
fn get_service_name() -> String {
    format!("{APP_NAME}.service")
}

/// Get filename for systemd service file
fn get_dot_service() -> String {
    let service = get_service_name();
    format!("/etc/systemd/system/{service}")
}

/// Create a systemd service file, with correct details
fn create_service_file(user_name: &str) -> Result<String, AppError> {
    let current_dir = env::current_dir()?.display().to_string();
    Ok(format!(
        r#"[Unit]
    Description={APP_NAME}
    After=network-online.target
    Wants=network-online.target
    StartLimitIntervalSec=0
    
    [Service]
    Environment="XDG_RUNTIME_DIR=/run/user/1000"
    Environment="WAYLAND_DISPLAY=wayland-1"
    ExecStart={current_dir}/{APP_NAME}
    WorkingDirectory={current_dir}
    SyslogIdentifier={APP_NAME}
    User={user_name}
    Group={user_name}
    Restart=always
    RestartSec=5

    [Install]
    WantedBy=multi-user.target
"#
    ))
}
/// If is sudo, and able to get a user name (which isn't root), install leafcast as a service
fn install_service() -> Result<(), AppError> {
    if let Some(user_name) = get_user_name() {
        tracing::info!("Create service file");
        let mut file = fs::File::create(get_dot_service())?;

        tracing::info!("Write unit text to file");
        file.write_all(create_service_file(&user_name)?.as_bytes())?;

        tracing::info!("Reload systemctl daemon");
        Command::new(SYSTEMCTL).arg("daemon-reload").output()?;

        let service_name = get_service_name();
        tracing::info!("Enable service");
        Command::new(SYSTEMCTL)
            .args(["enable", &service_name])
            .output()?;

        tracing::info!("Start service");
        Command::new(SYSTEMCTL)
            .args(["start", &service_name])
            .output()?;
    } else {
        tracing::error!("invalid user");
    }
    Ok(())
}

/// (un)install service via systemd
pub fn configure_systemd(arg: CliArg) -> Result<(), AppError> {
    check_sudo()?;
    match arg {
        CliArg::Install => {
            uninstall_service()?;
            install_service()?;
            tracing::info!("Installed service");
        }
        CliArg::Uninstall => {
            uninstall_service()?;
            tracing::info!("Uninstalled service");
        }
        _ => (),
    }
    Ok(())
}

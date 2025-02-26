use heartbeat::HeartBeat;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod app_env;
mod app_error;
mod heartbeat;
mod sysinfo;
mod systemd;
mod ws;
mod ws_messages;

use std::env::Args;

use app_env::AppEnv;
use app_error::AppError;
use sysinfo::SysInfo;
use systemd::configure_systemd;
use ws::open_connection;

/// Simple macro to create a new String, or convert from a &str to  a String - basically just gets rid of String::from() / .to_owned() etc
#[macro_export]
macro_rules! S {
    () => {
        String::new()
    };
    ($s:expr) => {
        String::from($s)
    };
}

#[macro_export]
/// Sleep for a given number of milliseconds, is an async fn.
/// If no parameter supplied, defaults to 1000ms
macro_rules! sleep {
    () => {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await
    };
    ($ms:expr) => {
        tokio::time::sleep(std::time::Duration::from_millis($ms)).await
    };
}

/// Simple macro to call `.clone()` on whatever is passed in
#[macro_export]
macro_rules! C {
    ($i:expr) => {
        $i.clone()
    };
}

fn close_signal() {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        std::process::exit(1);
    });
}

fn setup_tracing(app_envs: Option<&AppEnv>) {
    tracing_subscriber::fmt()
        .with_max_level(app_envs.map_or(tracing::Level::DEBUG, |i| i.log_level))
        .init();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CliArg {
    On,
    Off,
    Install,
    Uninstall,
    Help,
}

/// display cli argument information
fn display_arg_info() {
    println!(
        "\n{} v{}\n",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("--on   Turn screen on");
    println!("--off  Turn screen off");
    println!("-i     Install systemd service, requires running as SUDO");
    println!("-u     Uninstall systemd service, requires running as SUDO");
    println!("-h     Display this Help section\n");
}
/// Parse the command line arguments
fn parse_arg(args: Args) -> Option<CliArg> {
    match args.skip(1).take(1).collect::<String>().trim() {
        "-i" => Some(CliArg::Install),
        "-u" => Some(CliArg::Uninstall),
        "--on" => Some(CliArg::On),
        "--off" => Some(CliArg::Off),
        "-h" => Some(CliArg::Help),
        _ => None,
    }
}

/// Run the client, connect to WS as long running process
async fn run_as_client() -> Result<(), AppError> {
    let app_envs = AppEnv::get();
    setup_tracing(Some(&app_envs));
    close_signal();
    HeartBeat::start(&app_envs);
    open_connection(app_envs).await?;
    Ok(())
}

// todo!("read and env with on and off time, and use that to turnthe screen on and off rather than a cronjob";)
// if want to change, need to reload service?
async fn start() -> Result<(), AppError> {
    if let Some(arg) = parse_arg(std::env::args()) {
        match arg {
            CliArg::Install | CliArg::Uninstall => {
                setup_tracing(None);
                if let Err(e) = configure_systemd(arg) {
                    tracing::error!("{e:?}");
                }
            }
            CliArg::On => {
                setup_tracing(None);
                if let Err(e) = SysInfo::turn_on().await {
                    tracing::error!("{e:?}");
                }
            }
            CliArg::Off => {
                setup_tracing(None);
                if let Err(e) = SysInfo::turn_off().await {
                    tracing::error!("{e:?}");
                }
            }
            CliArg::Help => display_arg_info(),
        }
    } else {
        run_as_client().await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tokio::spawn(start()).await.ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use jiff::civil::Time;

    use crate::app_env::AppEnv;

    pub fn test_setup() -> AppEnv {
        AppEnv {
            log_level: tracing::Level::INFO,
            start_time: SystemTime::now(),
            ws_address: S!("ws_address"),
            ws_apikey: S!("ws_apikey"),
            ws_password: S!("ws_password"),
            time_on: Time::constant(8, 0, 0, 0),
            time_off: Time::constant(9, 0, 0, 0),
            ws_token_address: S!("ws_token_address"),
        }
    }
}

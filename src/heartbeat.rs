use jiff::{Zoned, civil::Time};

use crate::{app_env::AppEnv, sleep, sysinfo::SysInfo};
pub struct HeartBeat;

impl HeartBeat {
    pub fn start(app_env: &AppEnv) {
        let on = app_env.time_on;
        let off = app_env.time_off;
        tokio::spawn(async move {
            Self::spawn(on, off).await;
        });
    }

    async fn spawn(on: Time, off: Time) {
        let mut status = false;
        loop {
            let current_time = Zoned::now();
            if !status
                && current_time.hour() == on.hour()
                && current_time.minute() == on.minute()
                && current_time.second() == 0
            {
                match SysInfo::turn_on().await {
                    Ok(_) => {
                        status = true;
                    }
                    Err(e) => tracing::error!("{e:}"),
                }
            }
            if status
                && current_time.hour() == off.hour()
                && current_time.minute() == off.minute()
                && current_time.second() == 0
            {
                match SysInfo::turn_off().await {
                    Ok(_) => {
                        status = false;
                    }
                    Err(e) => tracing::error!("{e:}"),
                }
            }
            sleep!(250);
        }
    }
}

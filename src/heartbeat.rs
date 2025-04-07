use jiff::{Zoned, civil::Time};

use crate::{app_env::AppEnv, sleep, sysinfo::SysInfo};
pub struct HeartBeat;

impl HeartBeat {
    pub fn start(app_env: &AppEnv) {
        let time_on = app_env.time_on;
        let time_off = app_env.time_off;
        tokio::spawn(async move {
            Self::spawn(time_on, time_off).await;
        });
    }

    async fn spawn(on: Time, off: Time) {
        loop {
            let current_time = Zoned::now();
            if current_time.hour() == on.hour()
                && current_time.minute() == on.minute()
                && current_time.second() == 0
            {
                if let Err(e) = SysInfo::turn_on().await {
                    tracing::error!("{e:}");
                }
            }
            if current_time.hour() == off.hour()
                && current_time.minute() == off.minute()
                && current_time.second() == 0
            {
                if let Err(e) = SysInfo::turn_off().await {
                    tracing::error!("{e:}");
                }
            }
            sleep!(250);
        }
    }
}

use async_channel::Sender;
use jiff::{Zoned, civil::Time};

use crate::{C, app_env::AppEnv, message_handler::Msg, sleep};
pub struct Croner;

impl Croner {
    pub fn start(app_env: &AppEnv, tx: &Sender<Msg>) {
        let (on, off, tx) = (app_env.time_on, app_env.time_off, C!(tx));
        tokio::spawn(async move {
            Self::spawn(on, off, tx).await;
        });
    }

    async fn spawn(on: Time, off: Time, tx: Sender<Msg>) {
        loop {
            let current_time = Zoned::now();
            if current_time.hour() == on.hour()
                && current_time.minute() == on.minute()
                && current_time.second() == 0
            {
                tx.send(Msg::ScreenOn).await.ok();
            }
            if current_time.hour() == off.hour()
                && current_time.minute() == off.minute()
                && current_time.second() == 0
            {
                tx.send(Msg::ScreenOff).await.ok();
            }
            sleep!(250);
        }
    }
}

use std::env;

use log::{error, info, warn};
use strecken_info::Disruption;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    task::JoinHandle,
};

use crate::Database;

#[cfg(feature = "mastodon")]
pub mod mastodon;
#[cfg(feature = "mastodon")]
use crate::components::mastodon::MastodonSender;
#[cfg(feature = "telegram")]
pub mod telegram;
#[cfg(feature = "telegram")]
use crate::components::telegram::run_bot;

pub struct DisruptionInformation {
    pub disruption_id: i64,
    pub changed: bool,
    pub disruption: Disruption,
}

#[derive(Clone)]
pub struct Components {
    channels: Vec<UnboundedSender<DisruptionInformation>>,
}

impl Components {
    pub fn new(channels: Vec<UnboundedSender<DisruptionInformation>>) -> Self {
        Self { channels }
    }

    pub async fn by_env(database: Database) -> (Self, Vec<JoinHandle<()>>) {
        let mut channels: Vec<UnboundedSender<DisruptionInformation>> = Vec::new();
        let mut tasks = Vec::new();
        if let Ok(mastodon_url) = env::var("MASTODON_URL") {
            if !cfg!(feature = "mastodon") {
                panic!("You tried to enable mastodon but this binary was built without mastodon feature.");
            }
            let mastodon_access_token = env::var("MASTODON_ACCESS_TOKEN")
                .expect("Environment variable 'MASTODON_ACCESS_TOKEN' not set");
            #[cfg(feature = "mastodon")]
            {
                let (mastodon_sender, mastodon_receiver) =
                    unbounded_channel::<DisruptionInformation>();
                let mastodon = MastodonSender::new(
                    database.clone(),
                    mastodon_receiver,
                    mastodon_url,
                    mastodon_access_token,
                )
                .await;
                tasks.push(mastodon.start_polling());
                info!("Mastodon started");
                channels.push(mastodon_sender);
            }
        }

        if let Ok(bot_token) = env::var("TELEGRAM_BOT_TOKEN") {
            if !cfg!(feature = "telegram") {
                panic!("You tried to enable telegram but this binary was built without telegram feature.");
            }
            #[cfg(feature = "telegram")]
            {
                let (telegram_sender, telegram_receiver) =
                    unbounded_channel::<DisruptionInformation>();
                tasks.extend(run_bot(database, telegram_receiver, bot_token).await);
                info!("Telegram started");
                channels.push(telegram_sender);
            }
        }

        if channels.is_empty() {
            warn!("No distribution configured!");
        }

        (Self::new(channels), tasks)
    }

    pub fn push(
        &self,
        disruption_id: i64,
        changed: bool,
        disruption: Disruption,
    ) -> Result<(), String> {
        for channel in &self.channels {
            if let Err(e) = channel.send(DisruptionInformation {
                disruption_id,
                changed,
                disruption: disruption.clone(),
            }) {
                error!(
                    "Error while sending new disruption information: {e}. Error will be ignored."
                );
            };
        }
        Ok(())
    }
}

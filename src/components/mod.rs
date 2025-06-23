use std::env;

use log::{error, info, warn};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    data::{DataDisruptionInformation, AVAILABLE_DATA_SOURCES},
    Database,
};

#[cfg(feature = "mastodon")]
pub mod mastodon;
#[cfg(feature = "mastodon")]
use crate::components::mastodon::MastodonSender;
#[cfg(feature = "telegram")]
pub mod telegram;
#[cfg(feature = "telegram")]
use crate::components::telegram::run_bot;

pub enum ComponentType {
    #[cfg(feature = "telegram")]
    Telegram,
    #[cfg(feature = "mastodon")]
    Mastodon,
}

#[derive(Clone)]
pub struct Components {
    channels: Vec<UnboundedSender<DataDisruptionInformation>>,
}

impl Components {
    pub fn new(channels: Vec<UnboundedSender<DataDisruptionInformation>>) -> Self {
        Self { channels }
    }

    pub async fn by_env(database: Database) -> (Self, Vec<JoinHandle<()>>) {
        let mut channels: Vec<UnboundedSender<DataDisruptionInformation>> = Vec::new();
        let mut tasks = Vec::new();
        for data_source in AVAILABLE_DATA_SOURCES {
            if let Ok(mastodon_url) = env::var(format!("MASTODON_{data_source}_URL")) {
                let mastodon_access_token = env::var(format!(
                    "MASTODON_{data_source}_ACCESS_TOKEN"
                ))
                .expect("Environment variable 'MASTODON_{data_source}_ACCESS_TOKEN' not set");

                let (mastodon_sender, mastodon_receiver) =
                    unbounded_channel::<DataDisruptionInformation>();
                let mastodon = MastodonSender::new(
                    database.clone(),
                    mastodon_receiver,
                    mastodon_url,
                    mastodon_access_token,
                    data_source.to_string(),
                )
                .await;
                tasks.push(mastodon.start_polling());
                info!("Mastodon for data source {data_source} started");
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
                    unbounded_channel::<DataDisruptionInformation>();
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

    pub fn push(&self, disruption: DataDisruptionInformation) {
        for channel in &self.channels {
            if let Err(e) = channel.send(disruption.clone()) {
                error!(
                    "Error while sending new disruption information: {e}. Error will be ignored."
                );
            };
        }
    }
}

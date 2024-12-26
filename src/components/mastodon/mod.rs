use std::env;

use log::info;
use megalodon::{
    megalodon::{PostStatusInputOptions, PostStatusOutput},
    Megalodon,
};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};

use crate::{components::DisruptionInformation, tw::get_disruption_tw_word, Database};

mod format;

fn get_user_agent() -> String {
    format!("strecken-info-bot/{}", env!("CARGO_PKG_VERSION"))
}

fn limit_message(message: String, limit: usize) -> String {
    let chars = message.chars().collect::<Vec<char>>();
    if chars.len() <= limit {
        message
    } else {
        let reduced_chars = &chars[..limit - 4];
        format!("{}...", reduced_chars.iter().collect::<String>())
    }
}

pub struct MastodonSender {
    client: Box<dyn Megalodon + Send + Sync>,
    database: Database,
    receiver: UnboundedReceiver<DisruptionInformation>,
    max_status_characters: u32,
}

impl MastodonSender {
    pub fn create_client(
        mastodon_url: String,
        mastodon_access_token: String,
    ) -> Box<dyn Megalodon + Send + Sync> {
        megalodon::generator(
            megalodon::SNS::Mastodon,
            mastodon_url,
            Some(mastodon_access_token),
            Some(get_user_agent()),
        )
        .unwrap()
    }

    pub fn create_client_by_env() -> Box<dyn Megalodon + Send + Sync> {
        Self::create_client(
            env::var("MASTODON_URL").expect("Environment variable 'MASTODON_URL' not set"),
            env::var("MASTODON_ACCESS_TOKEN")
                .expect("Environment variable 'MASTODON_ACCESS_TOKEN' not set"),
        )
    }

    /// Create a new MastodonSender if mastodon is configured
    /// MastodonSender::new will return None if mastodon isn't configured
    pub async fn new(
        database: Database,
        receiver: UnboundedReceiver<DisruptionInformation>,
        mastodon_url: String,
        mastodon_access_token: String,
    ) -> Self {
        let client = Self::create_client(mastodon_url, mastodon_access_token);

        let mut sender = Self {
            client,
            database,
            receiver,
            max_status_characters: 0,
        };
        sender.fetch_max_status_characters().await;

        sender
    }

    /// This function fetches the instance configuration of the mastodon server and updates the max_characters value
    async fn fetch_max_status_characters(&mut self) {
        self.max_status_characters = self
            .client
            .get_instance()
            .await
            .expect("Error while fetching instance information")
            .json
            .configuration
            .statuses
            .max_characters;
    }

    /// Post a new status on mastodon with `status` as content in reply to a status with the id `in_reply_to_id`
    /// post_status returns the id of the created status
    async fn post_status(
        &self,
        status: String,
        in_reply_to_id: Option<String>,
        spoiler_text: Option<String>,
    ) -> String {
        let options = PostStatusInputOptions {
            in_reply_to_id,
            spoiler_text,
            language: Some("de".to_string()),
            ..Default::default()
        };
        let status = self
            .client
            .post_status(status, Some(&options))
            .await
            .expect("Error while sending mastodon status");
        if let PostStatusOutput::Status(status) = status.json {
            return status.id;
        }
        // I'm sure this case is not reachable because I don't send a ScheduledStatus
        panic!("Invalid mastodon post_status response: Not a PostStatusOutput::Status");
    }

    /// Send a disruption to mastodon
    /// `disruption_id` must contain the primary key of the disruption in the disruption table of the database
    /// `message` must contain the message that should be sent to mastodon
    /// the `message` string could be reduced cause of character limitations
    async fn send_disruption(&self, disruption_id: i32, disruption: DisruptionInformation) {
        let message = format::format(
            &disruption.disruption,
            &disruption.changes,
            disruption.update,
        );
        let message = limit_message(message, self.max_status_characters as usize);
        let connection = self.database.get_connection().await.unwrap();
        // toot_id will contain the primary key of the toot in the toot table
        // status_id will contain the id of the last mastodon status associated with the disruption
        let (toot_id, status_id) = match connection
            .query_opt(
                "SELECT id, status_id FROM mastodon_toot WHERE disruption_id=$1",
                &[&disruption_id],
            )
            .await
            .unwrap()
        {
            Some(row) => (Some(row.get::<_, i32>(0)), Some(row.get(1))),
            None => (None, None),
        };

        let trigger_word = match env::var("MASTODON_TRIGGER_WARNINGS") {
            Ok(tws) => {
                let tws = tws.split(',').collect::<Vec<&str>>();
                get_disruption_tw_word(&disruption.disruption, &tws)
            }
            _ => None,
        };

        let new_status_id = self
            .post_status(
                message,
                status_id,
                trigger_word.map(|tw| format!("TW: {tw}")),
            )
            .await;
        if let Some(toot_id) = toot_id {
            connection
                .execute(
                    "UPDATE mastodon_toot SET status_id=$1 WHERE id=$2",
                    &[&new_status_id, &toot_id],
                )
                .await
                .unwrap();
        } else {
            connection
                .execute(
                    "INSERT INTO mastodon_toot(status_id, disruption_id) VALUES ($1,$2)",
                    &[&new_status_id, &disruption_id],
                )
                .await
                .unwrap();
        }
    }

    /// start the polling on the receiver async
    pub fn start_polling(mut self) -> JoinHandle<()> {
        info!("Mastodon sender is ready");
        tokio::spawn(async move {
            while let Some(disruption) = self.receiver.recv().await {
                self.send_disruption(disruption.disruption_id, disruption)
                    .await;
            }
        })
    }
}

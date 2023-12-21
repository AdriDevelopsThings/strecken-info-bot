use std::env;

use log::info;
use megalodon::{
    megalodon::{PostStatusInputOptions, PostStatusOutput},
    Megalodon,
};
use r2d2_sqlite::rusqlite::params;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::Database;

fn get_user_agent() -> String {
    format!("strecken-info-telegram/{}", env!("CARGO_PKG_VERSION"))
}

fn limit_message(message: String, limit: usize) -> String {
    if message.len() <= limit {
        message
    } else {
        format!("{}...", &message[..limit - 4])
    }
}

pub struct MastodonSender {
    client: Box<dyn Megalodon + Send + Sync>,
    database: Database,
    receiver: UnboundedReceiver<(i64, String)>,
    max_status_characters: u32,
}

impl MastodonSender {
    pub fn create_client() -> Option<Box<dyn Megalodon + Send + Sync>> {
        let mastodon_url = match env::var("MASTODON_URL") {
            Ok(url) => url,
            Err(_) => return None,
        };
        let mastodon_access_token =
            env::var("MASTODON_ACCESS_TOKEN").expect("No 'MASTODON_ACCESS_TOKEN' set");

        Some(megalodon::generator(
            megalodon::SNS::Mastodon,
            mastodon_url,
            Some(mastodon_access_token),
            Some(get_user_agent()),
        ))
    }

    /// Create a new MastodonSender if mastodon is configured
    /// MastodonSender::new will return None if mastodon isn't configured
    pub async fn new(
        database: Database,
        receiver: UnboundedReceiver<(i64, String)>,
    ) -> Option<Self> {
        let client = match Self::create_client() {
            Some(client) => client,
            None => return None,
        };

        let mut sender = Self {
            client,
            database,
            receiver,
            max_status_characters: 0,
        };
        sender.fetch_max_status_characters().await;

        Some(sender)
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
    async fn post_status(&self, status: String, in_reply_to_id: Option<String>) -> String {
        let options = PostStatusInputOptions {
            in_reply_to_id,
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
    async fn send_disruption(&self, disruption_id: i64, message: String) {
        let message = limit_message(message, self.max_status_characters as usize);
        let connection = self.database.get_connection().unwrap();
        // toot_id will contain the primary key of the toot in the toot table
        // status_id will contain the id of the last mastodon status associated with the disruption
        let (toot_id, status_id) = match connection.query_row(
            "SELECT id, status_id FROM toots WHERE disruption_id=?",
            params![disruption_id],
            |row| Ok((row.get::<usize, i64>(0)?, row.get::<usize, String>(1)?)),
        ) {
            Ok((toot_id, status_id)) => (Some(toot_id), Some(status_id)),
            Err(_) => (None, None),
        };

        let new_status_id = self.post_status(message, status_id).await;
        if let Some(toot_id) = toot_id {
            connection
                .execute(
                    "UPDATE toots SET status_id=? WHERE id=?",
                    params![new_status_id, toot_id],
                )
                .unwrap();
        } else {
            connection
                .execute(
                    "INSERT INTO toots(status_id, disruption_id) VALUES (?,?)",
                    params![new_status_id, disruption_id],
                )
                .unwrap();
        }
    }

    /// start the polling on the receiver async
    pub fn start_polling(mut self) {
        info!("Mastodon sender is ready");
        tokio::spawn(async move {
            while let Some((id, message)) = self.receiver.recv().await {
                self.send_disruption(id, message).await;
            }
        });
    }
}

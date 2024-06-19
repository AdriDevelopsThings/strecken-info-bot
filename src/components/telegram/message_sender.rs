use std::{error, sync::Arc};

use log::{error, info};
use r2d2_sqlite::rusqlite::params;
use telexide::{
    api::{types::SendMessage, API},
    Error, TelegramError,
};
use tokio::sync::{mpsc::UnboundedReceiver, Semaphore};

use crate::{
    components::{
        telegram::{format, user::User},
        DisruptionInformation,
    },
    database::Database,
    filter::DisruptionFilter,
};

const MAX_REQUESTS_AT_THE_SAME_TIME: usize = 16;
const FILTERS: &[DisruptionFilter] = &[DisruptionFilter::TooLongDisruption { days: 7 }];

#[derive(Clone)]
pub struct MessageSender {
    api_client: Arc<Box<dyn API + Send>>,
    database: Database,
    semaphore: Arc<Semaphore>,
}

impl MessageSender {
    pub fn new(api_client: Arc<Box<dyn API + Send>>, database: Database) -> Self {
        Self {
            api_client,
            database,
            semaphore: Arc::new(Semaphore::new(MAX_REQUESTS_AT_THE_SAME_TIME)),
        }
    }

    async fn send_message(&self, chat_id: i64, message: String) {
        let _permit = self.semaphore.acquire().await.unwrap();
        let mut message = SendMessage::new(chat_id.into(), message);
        message.set_parse_mode(telexide::model::ParseMode::HTML);
        if let Err(e) = self.api_client.send_message(message).await {
            if let Error::Telegram(TelegramError::APIResponseError(api_response)) = e {
                if api_response == "Forbidden: bot was blocked by the user"
                    || api_response == "Forbidden: user is deactivated"
                    || api_response == "Forbidden: bot was kicked from the supergroup chat"
                {
                    self.database
                        .get_connection()
                        .unwrap()
                        .execute("DELETE FROM user WHERE chat_id=?", params![chat_id])
                        .unwrap();
                } else {
                    error!("Api error while sending message to telegram: {api_response}");
                }
            } else {
                error!("Error while sending message to telegram: {e}");
            }
        }
    }

    pub async fn start_polling(
        &self,
        mut receiver: UnboundedReceiver<DisruptionInformation>,
    ) -> Result<(), Box<dyn error::Error>> {
        info!("Telegram sender is ready");
        while let Some(disruption) = receiver.recv().await {
            if !DisruptionFilter::filters(FILTERS, &disruption.disruption) {
                continue;
            }

            let message = format::format(&disruption.disruption, disruption.changed);

            let connection = self.database.get_connection().unwrap();
            let mut statement = connection.prepare(
                "SELECT id, chat_id, trigger_warning_list, show_planned_disruptions FROM user",
            )?;
            let users = statement
                .query_map([], User::from_row)?
                .collect::<Result<Vec<User>, r2d2_sqlite::rusqlite::Error>>()?;

            for user in users {
                let message = if let Some(trigger) = user.is_trigger(&message) {
                    format!("TW: {trigger}\n<span class=\"tg-spoiler\">{message}</span>")
                } else {
                    message.clone()
                };

                let cloned_self = self.clone();
                tokio::spawn(async move {
                    MessageSender::send_message(&cloned_self, user.chat_id, message).await;
                });
            }
        }
        Ok(())
    }
}

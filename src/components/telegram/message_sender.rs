use std::{error, sync::Arc};

use log::{error, info};
use telexide::{
    api::{types::SendMessage, API},
    Error, TelegramError,
};
use tokio::sync::{mpsc::UnboundedReceiver, Semaphore};

use crate::{
    components::{telegram::user::User, ComponentType},
    data::DataDisruptionInformation,
    database::Database,
};

const COMPONENT_TYPE: ComponentType = ComponentType::Telegram;
const MAX_REQUESTS_AT_THE_SAME_TIME: usize = 16;

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
                        .await
                        .unwrap()
                        .execute("DELETE FROM telegram_user WHERE chat_id=$1", &[&chat_id])
                        .await
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
        mut receiver: UnboundedReceiver<DataDisruptionInformation>,
    ) -> Result<(), Box<dyn error::Error>> {
        info!("Telegram sender is ready");
        while let Some(disruption) = receiver.recv().await {
            let message = disruption.format(COMPONENT_TYPE);

            let connection = self.database.get_connection().await.unwrap();
            let statement = connection
                .prepare("SELECT id, chat_id, trigger_warnings, show_planned_disruptions, filters, one_filter_enough FROM telegram_user")
                .await?;
            let users = connection
                .query(&statement, &[])
                .await?
                .iter()
                .map(User::from_row)
                .collect::<Result<Vec<User>, serde_json::Error>>()?;

            for user in users {
                if user
                    .is_filtered(disruption.disruption.as_ref(), &self.database.trassenfinder)
                    .await
                {
                    continue;
                }

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

use std::sync::Arc;

use log::{error, info};
use r2d2_sqlite::rusqlite::params;
use telexide::{
    api::{types::SendMessage, API},
    Error, TelegramError,
};
use tokio::sync::{mpsc::UnboundedReceiver, Semaphore};

use crate::database::Database;

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

    async fn send_message(&self, (chat_id, message): (i64, String)) {
        let _permit = self.semaphore.acquire().await.unwrap();
        let mut message = SendMessage::new(chat_id.into(), message);
        message.set_parse_mode(telexide::model::ParseMode::HTML);
        if let Err(e) = self.api_client.send_message(message).await {
            if let Error::Telegram(TelegramError::APIResponseError(api_response)) = e {
                if api_response == "Forbidden: bot was blocked by the user"
                    || api_response == "Forbidden: user is deactivated"
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

    pub async fn start_polling(&self, mut receiver: UnboundedReceiver<(i64, String)>) {
        info!("Telegram sender is ready");
        while let Some(message) = receiver.recv().await {
            let cloned_self = self.clone();
            tokio::spawn(async move {
                MessageSender::send_message(&cloned_self, message).await;
            });
        }
    }
}

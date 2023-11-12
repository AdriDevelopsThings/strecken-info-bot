use std::env;

use log::error;
use r2d2_sqlite::rusqlite::params;
use telexide::{
    api::types::SendMessage, create_framework, prelude::ClientBuilder, Client, Error, TelegramError,
};
use tokio::sync::mpsc::UnboundedReceiver;
use typemap_rev::TypeMapKey;

use info::*;
use subscribe::*;
use tw::*;

use crate::database::Database;

mod info;
mod subscribe;
mod tw;

struct HashMapDatabase;
impl TypeMapKey for HashMapDatabase {
    type Value = Database;
}

pub fn create_client() -> Client {
    let token = env::var("TELEGRAM_BOT_TOKEN")
        .expect("No TELEGRAM_BOT_TOKEN environment variable supplied");
    ClientBuilder::new()
        .set_token(&token)
        .set_framework(create_framework!(
            "strecken-info-bot",
            start,
            unsubscribe,
            version,
            git,
            feedback,
            tw
        ))
        .build()
}

pub async fn run_bot(
    database: Database,
    mut receiver: UnboundedReceiver<(i64, String)>,
) -> telexide::Result<()> {
    let client = create_client();
    let api_client = client.api_client.clone();

    {
        let mut data = client.data.write();
        data.insert::<HashMapDatabase>(database.clone());
    }

    tokio::spawn(async move {
        while let Some((chat_id, message)) = receiver.recv().await {
            let mut message = SendMessage::new(chat_id.into(), message);
            message.set_parse_mode(telexide::model::ParseMode::HTML);
            if let Err(e) = api_client.send_message(message).await {
                if let Error::Telegram(TelegramError::APIResponseError(api_response)) = e {
                    if api_response == "Forbidden: bot was blocked by the user" {
                        database
                            .get_connection()
                            .unwrap()
                            .execute("DELETE FROM user WHERE chat_id=?", params![chat_id])
                            .unwrap();
                    } else {
                        error!("Api error while sending message to telegram: ${api_response}");
                    }
                } else {
                    error!("Error while sending message to telegram: {e}");
                }
            }
        }
    });

    println!("Telegram bot started");
    client.start().await
}

use std::env;

use log::error;
use telexide::{api::types::SendMessage, create_framework, prelude::ClientBuilder};
use tokio::sync::mpsc::UnboundedReceiver;
use typemap_rev::TypeMapKey;

use info::*;
use subscribe::*;

use crate::database::Database;

mod info;
mod subscribe;

struct HashMapDatabase;
impl TypeMapKey for HashMapDatabase {
    type Value = Database;
}

pub async fn run_bot(
    database: Database,
    mut receiver: UnboundedReceiver<(i64, String)>,
) -> telexide::Result<()> {
    let token = env::var("TELEGRAM_BOT_TOKEN")
        .expect("No TELEGRAM_BOT_TOKEN environment variable supplied");
    let client = ClientBuilder::new()
        .set_token(&token)
        .set_framework(create_framework!(
            "strecken-info-bot",
            start,
            unsubscribe,
            version,
            git
        ))
        .build();
    let api_client = client.api_client.clone();
    tokio::spawn(async move {
        while let Some((chat_id, message)) = receiver.recv().await {
            let mut message = SendMessage::new(chat_id.into(), message);
            message.set_parse_mode(telexide::model::ParseMode::HTML);
            if let Err(e) = api_client.send_message(message).await {
                error!("Error while sending message to telegram: {e}");
            }
        }
    });

    {
        let mut data = client.data.write();
        data.insert::<HashMapDatabase>(database);
    }

    println!("Telegram bot started");
    client.start().await
}

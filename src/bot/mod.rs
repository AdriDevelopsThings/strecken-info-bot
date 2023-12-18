use std::env;

use log::info;
use telexide::{create_framework, prelude::ClientBuilder, Client};
use tokio::sync::mpsc::UnboundedReceiver;
use typemap_rev::TypeMapKey;

use info::*;
use planned::*;
use subscribe::*;
use tw::*;

use crate::{bot::message_sender::MessageSender, database::Database};

mod info;
mod message_sender;
mod planned;
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
            tw,
            planned
        ))
        .build()
}

pub async fn run_bot(
    database: Database,
    receiver: UnboundedReceiver<(i64, String)>,
) -> telexide::Result<()> {
    let client = create_client();
    let api_client = client.api_client.clone();

    {
        let mut data = client.data.write();
        data.insert::<HashMapDatabase>(database.clone());
    }
    let message_sender = MessageSender::new(api_client, database);
    tokio::spawn(async move {
        message_sender.start_polling(receiver).await;
    });

    info!("Telegram bot started");
    client.start().await
}

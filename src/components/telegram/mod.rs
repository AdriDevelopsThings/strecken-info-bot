use telexide::{create_framework, prelude::ClientBuilder, Client};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use typemap_rev::TypeMapKey;

use info::*;
use subscribe::*;
use tw::*;

use crate::Database;

use self::message_sender::MessageSender;

use super::DisruptionInformation;

mod format;
mod info;
mod message_sender;
mod subscribe;
mod tw;
mod user;

struct HashMapDatabase;
impl TypeMapKey for HashMapDatabase {
    type Value = Database;
}

pub fn create_client(bot_token: String) -> Client {
    ClientBuilder::new()
        .set_token(&bot_token)
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
    receiver: UnboundedReceiver<DisruptionInformation>,
    bot_token: String,
) -> [JoinHandle<()>; 2] {
    let client = create_client(bot_token);
    let api_client = client.api_client.clone();

    {
        let mut data = client.data.write();
        data.insert::<HashMapDatabase>(database.clone());
    }
    let message_sender = MessageSender::new(api_client, database);
    [
        tokio::spawn(async move {
            message_sender
                .start_polling(receiver)
                .await
                .expect("Error while running message sender");
        }),
        tokio::spawn(async move {
            client
                .start()
                .await
                .expect("Error while running telegram bot");
        }),
    ]
}

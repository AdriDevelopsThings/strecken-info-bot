use std::env;

use log::info;
use r2d2_sqlite::rusqlite::params;
use telexide::{
    api::types::SendMessage,
    create_framework,
    prelude::{command, ClientBuilder, CommandResult, Context, Message},
};
use tokio::sync::mpsc::UnboundedReceiver;
use typemap_rev::TypeMapKey;

use crate::database::Database;

struct HashMapDatabase;
impl TypeMapKey for HashMapDatabase {
    type Value = Database;
}

#[command(description = "Start of strecken-info")]
async fn start(context: Context, message: Message) -> CommandResult {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().unwrap();
    connection
        .execute(
            "INSERT INTO user(chat_id) VALUES(?) ON CONFLICT(chat_id) DO NOTHING",
            params![message.chat.get_id()],
        )
        .unwrap();
    info!(
        "New user {} subscribed",
        message
            .from
            .map(|user| format!(
                "{} {} ({})",
                user.first_name,
                user.last_name.unwrap_or_else(|| "None".to_string()),
                user.username.unwrap_or_else(|| "None".to_string())
            ))
            .unwrap_or_default()
    );
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            "You will now receive notifications about disruptions",
        ))
        .await?;
    Ok(())
}

#[command(description = "Unsubscribe for disruption updates")]
async fn unsubscribe(context: Context, message: Message) -> CommandResult {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().unwrap();
    connection
        .execute(
            "DELETE FROM user WHERE chat_id=?",
            params![message.chat.get_id()],
        )
        .unwrap();
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            "You will now don't receive any notifications about disruptions",
        ))
        .await?;
    Ok(())
}

pub async fn run_bot(
    database: Database,
    mut receiver: UnboundedReceiver<(i64, String)>,
) -> telexide::Result<()> {
    let token = env::var("TELEGRAM_BOT_TOKEN")
        .expect("No TELEGRAM_BOT_TOKEN environment variable supplied");
    let client = ClientBuilder::new()
        .set_token(&token)
        .set_framework(create_framework!("strecken-info-bot", start, unsubscribe))
        .build();
    let api_client = client.api_client.clone();
    tokio::spawn(async move {
        loop {
            let (chat_id, message) = receiver.recv().await.unwrap();
            let mut message = SendMessage::new(chat_id.into(), message);
            message.set_parse_mode(telexide::model::ParseMode::HTML);
            api_client.send_message(message).await.unwrap();
        }
    });

    {
        let mut data = client.data.write();
        data.insert::<HashMapDatabase>(database);
    }

    println!("Telegram bot started");
    client.start().await
}

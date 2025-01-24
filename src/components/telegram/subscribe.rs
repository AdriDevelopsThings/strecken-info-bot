use log::info;
use telexide::{api::types::SendMessage, prelude::*};

use crate::database::DbConnection;

use super::HashMapDatabase;

pub async fn subscribe_user(connection: &DbConnection<'_>, chat_id: i64) -> i32 {
    connection
        .query_one(
            "INSERT INTO telegram_user(chat_id) VALUES($1) ON CONFLICT(chat_id) DO UPDATE SET chat_id=EXCLUDED.chat_id RETURNING id",
            &[&chat_id],
        )
        .await
        .unwrap()
        .get(0)
}

#[command(description = "Starte den Bot und abonniere Störungsmeldungen")]
async fn start(context: Context, message: Message) -> CommandResult {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().await.unwrap();
    subscribe_user(&connection, message.chat.get_id()).await;
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
            "Du erhälst nun Nachrichten über neue oder geänderte Störungen.",
        ))
        .await?;
    Ok(())
}

#[command(description = "Deabonniere Störungsmeldungen")]
async fn unsubscribe(context: Context, message: Message) -> CommandResult {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().await.unwrap();
    connection
        .execute(
            "DELETE FROM telegram_user WHERE chat_id=$1",
            &[&message.chat.get_id()],
        )
        .await
        .unwrap();
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            "Du erhälst nun keine Nachrichten mehr zu Störungen.",
        ))
        .await?;
    Ok(())
}

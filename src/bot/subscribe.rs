use log::info;
use r2d2::PooledConnection;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};
use telexide::{api::types::SendMessage, prelude::*};

use super::HashMapDatabase;

pub fn subscribe_user(connection: &PooledConnection<SqliteConnectionManager>, chat_id: i64) {
    connection
        .execute(
            "INSERT INTO user(chat_id) VALUES(?) ON CONFLICT(chat_id) DO NOTHING",
            params![chat_id],
        )
        .unwrap();
}

#[command(description = "Start this bot by subscribing")]
async fn start(context: Context, message: Message) -> CommandResult {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().unwrap();
    subscribe_user(&connection, message.chat.get_id());
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

#[command(description = "Unsubscribe from disruption updates")]
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

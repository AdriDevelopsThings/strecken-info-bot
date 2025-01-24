use log::{error, warn};
use std::{env, error::Error};
use telexide::{
    api::types::SendMessage,
    client::Context,
    macros::prepare_listener,
    model::{Message, MessageContent, Update, UpdateContent},
};

use super::{show_users, HashMapDatabase};

const TELEGRAM_MAX_MESSAGE_SIZE: usize = 4096;

#[prepare_listener]
pub async fn admin_callback(context: Context, update: Update) {
    if let UpdateContent::Message(message) = update.content {
        if let Err(e) = admin_message(context, message).await {
            error!("Error while calling admin_message: {e}");
        }
    }
}

async fn admin_message(context: Context, message: Message) -> Result<(), Box<dyn Error>> {
    // get admin user id
    let admin_user_id = match env::var("TELEGRAM_ADMIN_USER_ID") {
        Ok(v) => v.parse::<i64>()?,
        Err(_) => {
            warn!("Someone (id: {}) tried to use an admin command but `TELEGRAM_ADMIN_USER_ID` is not set.", message.from.map(|user| user.id.to_string()).unwrap_or_else(|| "<I don't know>".to_string()));
            return Ok(());
        }
    };

    // get message content
    let content = match message.content {
        MessageContent::Text {
            content,
            entities: _,
        } => content,
        _ => return Ok(()),
    };

    // check admin permissions of sender
    match message.from {
        Some(user) => {
            if user.id != admin_user_id {
                return Ok(());
            }
        }
        None => return Ok(()),
    };

    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();

    if content.as_str() == "/list_users" {
        let text = show_users(context.api.clone(), database).await;
        let lines = text.split('\n');
        let mut msg = String::new();
        for line in lines {
            if msg.len() + line.len() + 1 >= TELEGRAM_MAX_MESSAGE_SIZE {
                context
                    .api
                    .send_message(SendMessage::new(message.chat.get_id().into(), msg))
                    .await
                    .unwrap();
                msg = String::new();
            }
            msg += line;
            msg += "\n";
        }
        if !msg.is_empty() {
            context
                .api
                .send_message(SendMessage::new(message.chat.get_id().into(), msg))
                .await
                .unwrap();
        }
    }

    Ok(())
}

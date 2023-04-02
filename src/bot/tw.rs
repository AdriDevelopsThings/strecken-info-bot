use r2d2_sqlite::rusqlite::params;
use telexide::{api::types::SendMessage, prelude::*};

use crate::database::Database;

use super::{subscribe::subscribe_user, HashMapDatabase};

async fn send_tw_help(database: Database, context: &Context, message: &Message) -> CommandResult {
    let connection = database.get_connection().unwrap();
    let trigger_warnings: String = connection
        .query_row(
            "SELECT trigger_warning_list FROM user WHERE chat_id=?",
            params![message.chat.get_id()],
            |row| row.get(0),
        )
        .unwrap();
    let trigger_warnings = trigger_warnings
        .split(',')
        .filter(|r| !r.is_empty())
        .collect::<Vec<&str>>()
        .join(",");
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            format!(
                "Configure your trigger warnings by using /tw add your_triggering_word
You can remove a trigger warning by using /tw remove your_triggering_word
You also can remove all trigger warnings by using /tw clear
These trigger warnings are configured for your user: {trigger_warnings}.
All disruptions that contain a triggering word will be sent as spoilers to you."
            ),
        ))
        .await?;
    Ok(())
}

#[command(description = "Edit your comma sperated trigger warning list")]
async fn tw(context: Context, message: Message) -> CommandResult {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().unwrap();
    subscribe_user(&connection, message.chat.get_id());
    let message_text = message.get_text().unwrap();
    let args = message_text.split(' ').collect::<Vec<&str>>();

    let trigger_warnings: String = connection
        .query_row(
            "SELECT trigger_warning_list FROM user WHERE chat_id=?",
            params![message.chat.get_id()],
            |row| row.get(0),
        )
        .unwrap();
    let mut trigger_warnings = trigger_warnings
        .split(',')
        .map(|s| s.to_owned())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

    if args.len() == 1 {
        send_tw_help(database.clone(), &context, &message).await?;
    } else {
        match args[1].to_lowercase().as_str() {
            "help" => send_tw_help(database.clone(), &context, &message).await?,
            "add" => {
                if args.len() < 3 {
                    send_tw_help(database.clone(), &context, &message).await?;
                } else {
                    let word = args[2..].join(" ");
                    trigger_warnings.push(word.to_owned());
                    context
                        .api
                        .send_message(SendMessage::new(
                            message.chat.get_id().into(),
                            format!("I added '{word}' to your trigger warnings."),
                        ))
                        .await
                        .unwrap();
                }
            }
            "remove" => {
                if args.len() < 3 {
                    send_tw_help(database.clone(), &context, &message).await?;
                } else {
                    let word = args[2..].join(" ").to_lowercase();
                    trigger_warnings.retain(|w| w.to_lowercase() != word);
                    context
                        .api
                        .send_message(SendMessage::new(
                            message.chat.get_id().into(),
                            format!("I removed '{word}' from your trigger warnings."),
                        ))
                        .await
                        .unwrap();
                }
            }
            "clear" => {
                trigger_warnings.clear();
                context
                    .api
                    .send_message(SendMessage::new(
                        message.chat.get_id().into(),
                        "I removed all words from your trigger warnings.",
                    ))
                    .await
                    .unwrap();
            }
            _ => send_tw_help(database.clone(), &context, &message).await?,
        }

        connection
            .execute(
                "UPDATE user SET trigger_warning_list=? WHERE chat_id=?",
                params![trigger_warnings.join(","), message.chat.get_id()],
            )
            .unwrap();
    }
    Ok(())
}

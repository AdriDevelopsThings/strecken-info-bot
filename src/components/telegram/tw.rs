use telexide::{api::types::SendMessage, prelude::*};

use crate::database::Database;

use super::{subscribe::subscribe_user, HashMapDatabase};

async fn send_tw_help(database: Database, context: &Context, message: &Message) -> CommandResult {
    let connection = database.get_connection().await.unwrap();
    let trigger_warnings = connection
        .query_one(
            "SELECT trigger_warnings FROM telegram_user WHERE chat_id=$1",
            &[&message.chat.get_id()],
        )
        .await
        .unwrap()
        .get::<_, Vec<String>>(0)
        .join(", ");
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            format!(
                "Füge ein Wort zu den Trigger Warnings mit /tw add WORT hinzu.
Du kannst eine Trigger Warning mit /tw remove WORT wieder entfernen.
Mit /tw clear kannst du alle Trigger Warnings wieder entfernen.
Diese Trigger Warnings hast du bereits konfiguriert: {trigger_warnings}.
Alle Störungen, die ein Wort aus den konfigurierten Trigger Warnings enthalten, werden dir als Spoiler geschickt."
            ),
        ))
        .await?;
    Ok(())
}

#[command(description = "Bearbeite deine Trigger Warnings")]
async fn tw(context: Context, message: Message) -> CommandResult {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().await.unwrap();
    subscribe_user(&connection, message.chat.get_id()).await;
    let message_text = message.get_text().unwrap();
    let args = message_text.split(' ').collect::<Vec<&str>>();

    let mut trigger_warnings: Vec<String> = connection
        .query_one(
            "SELECT trigger_warnings FROM telegram_user WHERE chat_id=$1",
            &[&message.chat.get_id()],
        )
        .await
        .unwrap()
        .get(0);

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
                "UPDATE telegram_user SET trigger_warnings=$1 WHERE chat_id=$2",
                &[&trigger_warnings, &message.chat.get_id()],
            )
            .await
            .unwrap();
    }
    Ok(())
}

use std::env;

use telexide::{api::types::GetChat, model::Chat};

use crate::{components::telegram::create_client, Database};

pub async fn show_users(database: Database) {
    let client = create_client(
        env::var("TELEGRAM_BOT_TOKEN").expect("Environment variable 'TELEGRAM_BOT_TOKEN' not set"),
    );
    let connection = database.get_connection().await.unwrap();
    let rows = connection
        .query("SELECT chat_id FROM telegram_user", &[])
        .await
        .unwrap();
    println!("{} chats are currently registered:\n", rows.len());
    for row in rows {
        let chat = client
            .api_client
            .get_chat(GetChat {
                chat_id: row.get::<_, i64>(0).into(),
            })
            .await
            .unwrap();
        match chat {
            Chat::Private(chat) => println!(
                "User {} {} ({})",
                chat.first_name.unwrap_or_default(),
                chat.last_name.unwrap_or_default(),
                chat.username.unwrap_or_default()
            ),
            Chat::Group(chat) => println!("Group {}", chat.title),
            Chat::SuperGroup(chat) => println!(
                "Supergroup {} ({})",
                chat.title,
                chat.username.unwrap_or_default()
            ),
            Chat::Channel(chat) => println!(
                "Channel {} ({})",
                chat.title,
                chat.username.unwrap_or_default()
            ),
        }
    }
}

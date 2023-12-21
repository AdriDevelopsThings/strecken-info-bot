use telexide::{api::types::GetChat, model::Chat};

use crate::{create_client, Database};

pub async fn show_users(database: Database) {
    let client = create_client();
    let connection = database.get_connection().unwrap();
    let mut statement = connection.prepare("SELECT chat_id FROM user").unwrap();
    let users = statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<i64>, r2d2_sqlite::rusqlite::Error>>()
        .unwrap();
    println!("{} chats are currently registered:\n", users.len());
    for user in users {
        let chat = client
            .api_client
            .get_chat(GetChat {
                chat_id: user.into(),
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

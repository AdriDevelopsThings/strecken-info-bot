use std::sync::Arc;

use telexide::{
    api::{types::GetChat, API},
    model::Chat,
};

use crate::Database;

pub async fn show_users(api_client: Arc<Box<dyn API + Send>>, database: Database) -> String {
    let connection = database.get_connection().await.unwrap();
    let rows = connection
        .query("SELECT id, chat_id FROM telegram_user ORDER by id ASC", &[])
        .await
        .unwrap();

    let mut out = String::with_capacity(24 + (rows.len() * 12));
    out += &format!("{} chats are registered:\n", rows.len());

    for row in rows {
        let chat = api_client
            .get_chat(GetChat {
                chat_id: row.get::<_, i64>(1).into(),
            })
            .await
            .unwrap();
        out += &format!("\n{}: ", row.get::<_, i32>(0));
        match chat {
            Chat::Private(chat) => {
                let mut has_firstname = false;
                if let Some(firstname) = chat.first_name {
                    has_firstname = true;
                    out += &firstname;
                }

                if let Some(lastname) = chat.last_name {
                    if has_firstname {
                        out += " ";
                    } else {
                        // has_firstname is has_first_or_lastname now
                        has_firstname = true;
                    }
                    out += &lastname;
                }

                if let Some(username) = chat.username {
                    if has_firstname {
                        out += " "; // space does look better
                    }
                    out += "(";
                    out += &username;
                    out += ")";
                }
            }
            Chat::Group(chat) => {
                out += &format!("Group {}", chat.title);
            }
            Chat::SuperGroup(chat) => {
                out += &format!("SuperGroup {}", chat.title);
                if let Some(username) = chat.username {
                    out += "by ";
                    out += &username;
                }
            }
            Chat::Channel(chat) => {
                out += &format!("Channel {}", chat.title);
                if let Some(username) = chat.username {
                    out += "by ";
                    out += &username;
                }
            }
        };
    }

    out
}

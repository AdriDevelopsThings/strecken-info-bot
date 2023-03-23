use clap::Parser;
use dotenv::dotenv;
use std::{env, process};

use bot::create_client;
use database::Database;
use telexide::{api::types::GetChat, model::Chat};

mod bot;
mod database;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    show_users: bool,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();
    dotenv().ok();
    let database = Database::new(
        &env::var("SQLITE_PATH").expect("No SQLITE_PATH environment variable supplied"),
    )
    .unwrap();
    database.initialize().unwrap();
    let client = create_client();

    let args = Args::parse();

    if args.show_users {
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
    } else {
        eprintln!("No action supplied. Try --help to show all actions.");
        process::exit(1);
    }
}

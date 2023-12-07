use std::env;

use dotenv::dotenv;
use env_logger::Env;
use strecken_info_telegram::{run_bot, start_fetching, Database};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    dotenv().ok();
    let database = Database::new(
        &env::var("SQLITE_PATH").expect("No SQLITE_PATH environment variable supplied"),
    )
    .unwrap();
    database.initialize().unwrap();

    let (telegram_message_sender, telegram_message_receiver) =
        mpsc::unbounded_channel::<(i64, String)>();

    start_fetching(database.clone(), telegram_message_sender);
    run_bot(database, telegram_message_receiver).await.unwrap();
}

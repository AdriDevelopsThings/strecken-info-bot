use std::env;

use bot::run_bot;
use dotenv::dotenv;
use tokio::sync::mpsc;

use crate::{database::Database, fetcher::start_fetching};

mod bot;
mod database;
mod fetcher;
mod filter;
mod format;

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

    let (telegram_message_sender, telegram_message_receiver) =
        mpsc::unbounded_channel::<(i64, String)>();

    start_fetching(database.clone(), telegram_message_sender);
    run_bot(database, telegram_message_receiver).await.unwrap();
}

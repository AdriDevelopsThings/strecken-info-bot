use std::env;

use clap::Parser;
use dotenv::dotenv;
use env_logger::Env;
use strecken_info_telegram::{
    reset_disruptions, run_bot, show_users, start_cleaning, start_fetching, Database,
};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    show_users: bool,
    #[arg(short, long)]
    reset_disruptions: bool,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    dotenv().ok();
    let database = Database::new(
        &env::var("SQLITE_PATH").expect("No SQLITE_PATH environment variable supplied"),
    )
    .unwrap();
    database.initialize().unwrap();

    if args.show_users {
        show_users(database).await;
    } else if args.reset_disruptions {
        reset_disruptions(database).await;
    } else {
        let (telegram_message_sender, telegram_message_receiver) =
            mpsc::unbounded_channel::<(i64, String)>();

        start_fetching(database.clone(), telegram_message_sender);
        start_cleaning(database.clone());
        run_bot(database, telegram_message_receiver).await.unwrap();
    }
}

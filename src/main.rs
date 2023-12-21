use std::env;

use clap::Parser;
use dotenv::dotenv;
use env_logger::Env;
#[cfg(feature = "mastodon")]
use strecken_info_bot::clear_toots;
#[cfg(feature = "metrics")]
use strecken_info_bot::start_server;
#[cfg(feature = "mastodon")]
use strecken_info_bot::MastodonSender;
use strecken_info_bot::{
    reset_disruptions, run_bot, show_users, start_cleaning, start_fetching, Database,
};
use tokio::sync::mpsc::{self, UnboundedSender};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    show_users: bool,
    #[arg(short, long)]
    reset_disruptions: bool,
    #[cfg(feature = "mastodon")]
    #[arg(short, long)]
    clear_toots: bool,
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
    #[cfg(feature = "mastodon")]
    if args.clear_toots {
        clear_toots(database).await;
        return;
    }

    if args.show_users {
        show_users(database).await;
    } else if args.reset_disruptions {
        reset_disruptions(database).await;
    } else {
        let (telegram_message_sender, telegram_message_receiver) =
            mpsc::unbounded_channel::<(i64, String)>();
        let mut mastodon_message_sender: Option<UnboundedSender<(i64, String)>> = None;
        #[cfg(feature = "mastodon")]
        {
            let (mastodon_message_sender_, mastodon_message_receiver) =
                mpsc::unbounded_channel::<(i64, String)>();
            let mastodon = MastodonSender::new(database.clone(), mastodon_message_receiver).await;
            if let Some(mastodon) = mastodon {
                mastodon_message_sender = Some(mastodon_message_sender_);
                mastodon.start_polling();
            }
        }

        start_fetching(
            database.clone(),
            telegram_message_sender,
            mastodon_message_sender,
        );
        start_cleaning(database.clone());
        #[cfg(feature = "metrics")]
        start_server(database.clone()).await;
        run_bot(database, telegram_message_receiver).await.unwrap();
    }
}

use std::env;

use clap::Parser;
use dotenv::dotenv;
#[cfg(feature = "telegram")]
use strecken_info_bot::show_users;
#[cfg(feature = "metrics")]
use strecken_info_bot::start_server;
use strecken_info_bot::{
    reset_disruptions, start_fetching, Components, Database, TrassenfinderApi,
};
use tracing::error;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

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
    dotenv().ok();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriebr");

    let args = Args::parse();
    let mut database = Database::new(
        &env::var("POSTGRESQL_CONFIG").expect("No POSTGRESQL_CONFIG environment variable supplied"),
        None,
    )
    .await
    .expect("Error while connecting to database");
    database
        .initialize()
        .await
        .expect("Error while initializing database");

    #[cfg(feature = "telegram")]
    if args.show_users {
        show_users(database).await;
        return;
    }

    if args.reset_disruptions {
        reset_disruptions(database).await;
    } else {
        let trassenfinder = match TrassenfinderApi::new().await {
            Ok(t) => {
                t.start_reloading().await;
                Some(t)
            }
            Err(e) => {
                error!("Error while initializing trassenfinder api component: {e:?}");
                None
            }
        };
        database.trassenfinder = trassenfinder;

        let (components, tasks) = Components::by_env(database.clone()).await;

        start_fetching(database.clone(), components).await;

        #[cfg(feature = "metrics")]
        start_server(database.clone()).await;

        // exit the process if a worker panics
        let default_panic = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            default_panic(info);
            std::process::exit(1);
        }));

        futures::future::join_all(tasks).await;
    }
}

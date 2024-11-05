use std::{collections::HashMap, sync::Arc};

use admin::admin_callback;
use telexide::{create_framework, prelude::ClientBuilder, Client};
use tokio::{
    sync::{mpsc::UnboundedReceiver, Mutex},
    task::JoinHandle,
};
use typemap_rev::TypeMapKey;

use filter::*;
use info::*;
use subscribe::*;
use tw::*;

use crate::Database;

use self::message_sender::MessageSender;

use super::DisruptionInformation;

mod admin;
mod filter;
mod format;
mod info;
mod message_sender;
mod show_users;
mod subscribe;
mod tw;
mod user;

pub use filter::epsg_3857_to_epsg_4326;
pub use filter::Filter;
pub use show_users::show_users;

struct HashMapDatabase;
impl TypeMapKey for HashMapDatabase {
    type Value = Database;
}

#[derive(Clone)]
enum Expecting {
    Location,
    LocationRange { lon: f64, lat: f64 },
    RailwayManagement,
}

struct HashMapExpecting;
impl TypeMapKey for HashMapExpecting {
    type Value = Arc<Mutex<HashMap<i32, Expecting>>>;
}

pub fn create_client(bot_token: String) -> Client {
    ClientBuilder::new()
        .set_token(&bot_token)
        .set_framework(create_framework!(
            "strecken-info-bot",
            start,
            unsubscribe,
            version,
            git,
            feedback,
            tw,
            filter
        ))
        .add_handler_func(callback)
        .add_handler_func(admin_callback)
        .build()
}

pub async fn run_bot(
    database: Database,
    receiver: UnboundedReceiver<DisruptionInformation>,
    bot_token: String,
) -> [JoinHandle<()>; 2] {
    let client = create_client(bot_token);
    let api_client = client.api_client.clone();

    {
        let mut data = client.data.write();
        data.insert::<HashMapDatabase>(database.clone());
        data.insert::<HashMapExpecting>(Arc::new(Mutex::new(HashMap::new())));
    }
    let message_sender = MessageSender::new(api_client, database);
    [
        tokio::spawn(async move {
            message_sender
                .start_polling(receiver)
                .await
                .expect("Error while running message sender");
        }),
        tokio::spawn(async move {
            client
                .start()
                .await
                .expect("Error while running telegram bot");
        }),
    ]
}

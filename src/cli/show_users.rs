use std::env;

use crate::{
    components::telegram::{create_client, show_users as show_users_api},
    Database,
};

pub async fn show_users(database: Database) {
    let client = create_client(
        env::var("TELEGRAM_BOT_TOKEN").expect("Environment variable 'TELEGRAM_BOT_TOKEN' not set"),
    );
    println!("{}", show_users_api(client.api_client, database).await);
}

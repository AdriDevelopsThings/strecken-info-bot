use std::{env, error::Error, time::Duration};

use chrono::Utc;
use chrono_tz::Europe::Berlin;
use log::{debug, error, warn};
use r2d2_sqlite::rusqlite::params;
use tokio::{
    sync::mpsc::{self, UnboundedSender},
    time::interval,
};

use strecken_info::{geo_pos::request_disruptions, Disruption};

use crate::{
    database::Database,
    filter::Filter,
    format::{disruption_to_string, hash_disruption},
    user::User,
};

pub fn start_fetching(database: Database, telegram_message_sender: UnboundedSender<(i64, String)>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<Disruption>>();
    tokio::spawn(async move {
        let fetch_every: u64 = env::var("FETCH_EVERY_SECONDS")
            .unwrap_or_else(|_| "120".to_string())
            .parse()
            .expect("Error while parsing FETCH_EVERY_SECONDS environment variable");
        if fetch_every < 60 {
            warn!("It's not recommended to set FETCH_EVERY_SECONDS to a value below 60.");
        }
        let mut interval = interval(Duration::from_secs(fetch_every));
        loop {
            interval.tick().await;
            let now = Utc::now();
            let now = now.with_timezone(&Berlin).naive_local();
            let disruptions = match request_disruptions(now, now, 5000, 100, None).await {
                Ok(s) => s,
                Err(e) => {
                    error!(
                        "Error while fetching: {:?}, retrying in {fetch_every} seconds.",
                        e
                    );
                    continue;
                }
            };
            debug!("Fetched new disruptions");
            tx.send(disruptions).unwrap();
        }
    });

    tokio::spawn(async move {
        while let Some(s) = rx.recv().await {
            if let Err(e) = fetched(database.clone(), s, telegram_message_sender.clone()) {
                error!("Error while handling new fetch: {e}");
            }
        }
    });
}

async fn do_heartbeat() {
    if let Ok(heartbeat_url) = env::var("HEARTBEAT_URL") {
        reqwest::get(&heartbeat_url).await.unwrap();
        debug!("Heartbeat url {heartbeat_url} called");
    }
}

fn fetched(
    database: Database,
    disruptions: Vec<Disruption>,
    telegram_message_sender: UnboundedSender<(i64, String)>,
) -> Result<(), Box<dyn Error>> {
    let connection = database.get_connection()?;
    let filters = vec![Filter::Planned, Filter::TooLongDisruption { days: 7 }];
    let mut statement = connection.prepare("SELECT id, chat_id, trigger_warning_list FROM user")?;
    let users = statement
        .query_map([], User::from_row)?
        .collect::<Result<Vec<User>, r2d2_sqlite::rusqlite::Error>>()?;

    let mut changes = 0;
    for disruption in disruptions {
        let hash = hash_disruption(&disruption);
        let (send, changed) = match connection.query_row(
            "SELECT hash FROM disruption WHERE him_id=?",
            params![&disruption.id],
            |row| row.get::<usize, String>(0),
        ) {
            Ok(db_hash) => (hash != db_hash, true),
            Err(_) => (true, false),
        };
        if send {
            changes += 1;
            // Entry changed
            if Filter::filters(&filters, &disruption) {
                let message = disruption_to_string(&disruption, changed);
                // Send this disruption to users
                for user in &users {
                    let message = if let Some(trigger) = user.is_trigger(&message) {
                        format!("TW: {trigger}\n<span class=\"tg-spoiler\">{message}</span>")
                    } else {
                        message.clone()
                    };
                    telegram_message_sender.send((user.chat_id, message))?;
                }
            }
            connection.execute("INSERT INTO disruption(him_id, hash) VALUES(?, ?) ON CONFLICT(him_id) DO UPDATE SET hash=excluded.hash", params![&disruption.id, hash])?;
        }
    }
    debug!("{changes} disruptions found/changed");
    tokio::spawn(do_heartbeat());
    Ok(())
}

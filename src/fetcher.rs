use std::{env, error::Error, time::Duration};

use chrono::Utc;
use chrono_tz::Europe::Berlin;
use log::{error, info, warn};
use r2d2_sqlite::rusqlite::params;
use tokio::{
    sync::mpsc::{self, UnboundedSender},
    time::interval,
};

use strecken_info::{geo_pos::request_disruptions, Disruption};

use crate::{database::Database, filter::Filter, format::disruption_to_string};

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
            info!("Fetched new disruptions");
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

fn fetched(
    database: Database,
    disruptions: Vec<Disruption>,
    telegram_message_sender: UnboundedSender<(i64, String)>,
) -> Result<(), Box<dyn Error>> {
    let connection = database.get_connection()?;
    let filters = vec![Filter::PrioFilter { min: 30 }, Filter::PlannedFilter];
    let mut statement = connection.prepare("SELECT chat_id FROM user")?;
    let users = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<i64>, r2d2_sqlite::rusqlite::Error>>()?;

    let mut changes = 0;
    for disruption in disruptions {
        let message = disruption_to_string(&disruption);
        let hash = format!("{:x}", md5::compute(message.as_bytes()));
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
                let message = match changed {
                    true => "UPDATE: ".to_string(),
                    false => String::new(),
                } + message.as_str();
                // Send this disruption to users
                for user in &users {
                    telegram_message_sender.send((*user, message.clone()))?;
                }
            }
            connection.execute("INSERT INTO disruption(him_id, hash) VALUES(?, ?) ON CONFLICT(him_id) DO UPDATE SET hash=excluded.hash", params![&disruption.id, hash])?;
        }
    }
    info!("{changes} disruptions found/changed");
    Ok(())
}

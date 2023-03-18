use std::time::Duration;

use chrono::Utc;
use chrono_tz::Europe::Berlin;
use log::info;
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
        let mut interval = interval(Duration::from_secs(120));
        loop {
            interval.tick().await;
            let now = Utc::now();
            let now = now.with_timezone(&Berlin).naive_local();
            let disruptions = request_disruptions(now, now, 5000, 100, None)
                .await
                .unwrap();
            info!("Fetched new disruptions");
            tx.send(disruptions).unwrap();
        }
    });

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(s) => fetched(database.clone(), s, telegram_message_sender.clone()),
                None => continue,
            };
        }
    });
}

fn fetched(
    database: Database,
    disruptions: Vec<Disruption>,
    telegram_message_sender: UnboundedSender<(i64, String)>,
) {
    let connection = database.get_connection().unwrap();
    let filters = vec![Filter::PrioFilter { min: 30 }, Filter::PlannedFilter];
    let mut statement = connection.prepare("SELECT chat_id FROM user").unwrap();
    let users = statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Vec<Result<i64, r2d2_sqlite::rusqlite::Error>>>()
        .into_iter()
        .map(|r| r.unwrap())
        .collect::<Vec<i64>>();

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
            connection.execute("INSERT INTO disruption(him_id, hash) VALUES(?, ?) ON CONFLICT(him_id) DO UPDATE SET hash=excluded.hash", params![&disruption.id, hash]).unwrap();
            if Filter::filters(&filters, &disruption) {
                let message = match changed {
                    true => "UPDATE: ".to_string(),
                    false => String::new(),
                } + message.as_str();
                // Send this disruption to users
                for user in &users {
                    telegram_message_sender
                        .send((*user, message.clone()))
                        .unwrap();
                }
            }
        }
    }
    info!("{changes} disruptions found/changed");
}

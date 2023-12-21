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
    filter::{DisruptionFilter, UserFilter},
    format::{format_hash, format_telegram},
    user::User,
};

#[cfg(feature = "mastodon")]
use crate::format::format_mastodon;

pub fn start_fetching(
    database: Database,
    telegram_message_sender: UnboundedSender<(i64, String)>,
    mastodon_message_sender: Option<UnboundedSender<(i64, String)>>,
) {
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
            let mut disruptions = match request_disruptions(now, now, 5000, 100, None).await {
                Ok(s) => s,
                Err(e) => {
                    error!(
                        "Error while fetching: {:?}, retrying in {fetch_every} seconds.",
                        e
                    );
                    continue;
                }
            };
            // handle not planned disruptions first because the api could run into TooManyRequest errors
            disruptions.sort_by(|a, _| match a.planned {
                true => std::cmp::Ordering::Greater,
                false => std::cmp::Ordering::Less,
            });
            debug!("Fetched new disruptions");
            tx.send(disruptions).unwrap();
        }
    });

    tokio::spawn(async move {
        while let Some(s) = rx.recv().await {
            if let Err(e) = fetched(
                database.clone(),
                s,
                telegram_message_sender.clone(),
                mastodon_message_sender.clone(),
            ) {
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
    mastodon_message_sender: Option<UnboundedSender<(i64, String)>>,
) -> Result<(), Box<dyn Error>> {
    let connection = database.get_connection()?;
    let filters = vec![DisruptionFilter::TooLongDisruption { days: 7 }];
    #[cfg(feature = "mastodon")]
    let mastodon_filters = vec![
        DisruptionFilter::TooLongDisruption { days: 7 },
        DisruptionFilter::NotPlanned,
    ];
    let user_filters = vec![UserFilter::Planned];
    let mut statement = connection
        .prepare("SELECT id, chat_id, trigger_warning_list, show_planned_disruptions FROM user")?;
    let users = statement
        .query_map([], User::from_row)?
        .collect::<Result<Vec<User>, r2d2_sqlite::rusqlite::Error>>()?;

    let mut changes = 0;
    for disruption in disruptions {
        let hash = format_hash(&disruption);
        let db_disruption = connection.query_row(
            "SELECT id, hash FROM disruption WHERE him_id=?",
            params![&disruption.id],
            |row| Ok((row.get::<usize, i64>(0)?, row.get::<usize, String>(1)?)),
        );
        let (send, changed, disruption_id) = match db_disruption {
            Ok((disruption_id, db_hash)) => (hash != db_hash, true, Some(disruption_id)),
            Err(_) => (true, false, None),
        };
        if send {
            changes += 1;
            // Entry changed
            if DisruptionFilter::filters(&filters, &disruption) {
                let message = format_telegram(&disruption, changed);
                // Send this disruption to users
                for user in &users {
                    if !UserFilter::filters(&user_filters, &disruption, user) {
                        continue;
                    }
                    let message = if let Some(trigger) = user.is_trigger(&message) {
                        format!("TW: {trigger}\n<span class=\"tg-spoiler\">{message}</span>")
                    } else {
                        message.clone()
                    };
                    telegram_message_sender.send((user.chat_id, message))?;
                }
            }
            let start_time_sql = disruption
                .start_date
                .and_time(disruption.start_time)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            let end_time_sql = disruption
                .end_date
                .and_time(disruption.end_time)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            connection.execute(
                "INSERT INTO disruption(him_id, hash, start_time, end_time) VALUES(?, ?, ?, ?)
                ON CONFLICT(him_id) DO UPDATE
                SET hash=excluded.hash,
                    start_time=excluded.start_time,
                    end_time=excluded.end_time",
                params![&disruption.id, hash, start_time_sql, end_time_sql],
            )?;
            #[cfg(feature = "mastodon")]
            {
                if let Some(ref mastodon_message_sender) = mastodon_message_sender {
                    if DisruptionFilter::filters(&mastodon_filters, &disruption) {
                        let disruption_id =
                            disruption_id.unwrap_or_else(|| connection.last_insert_rowid());
                        mastodon_message_sender
                            .send((disruption_id, format_mastodon(&disruption, changed)))?;
                    }
                }
            }
        }
    }
    debug!("{changes} disruptions found/changed");
    tokio::spawn(do_heartbeat());
    Ok(())
}

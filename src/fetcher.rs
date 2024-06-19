use std::{env, error::Error};

use log::{debug, error};
use r2d2_sqlite::rusqlite::params;
use strecken_info::{
    disruptions::{request_disruptions, Disruption},
    filter::DisruptionsFilter,
    revision::RevisionContext,
};
use tokio::sync::mpsc;

use crate::{database::Database, format::hash::format_hash, Components};

pub fn start_fetching(database: Database, components: Components) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<Disruption>>();
    tokio::spawn(async move {
        let mut revision_ctx = RevisionContext::connect()
            .await
            .expect("Error while opening websocket");
        let mut revision = revision_ctx
            .get_first_revision()
            .await
            .expect("Error while getting first revision");

        'fetch: loop {
            let disruptions =
                match request_disruptions(DisruptionsFilter::default(), revision).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!(
                            "Error while fetching disruptions: {:?}, retrying in 10 seconds.",
                            e
                        );
                        continue 'fetch;
                    }
                };

            debug!("Fetched new disruptions");
            tx.send(disruptions).unwrap();
            revision = revision_ctx
                .wait_for_new_revision_filtered(true)
                .await
                .expect("Error while getting new revision");
        }
    });

    tokio::spawn(async move {
        while let Some(s) = rx.recv().await {
            if let Err(e) = fetched(database.clone(), s, components.clone()) {
                error!("Error while handling new fetch: {e}");
            }
        }
    });
}

async fn do_heartbeat() {
    if let Ok(heartbeat_url) = env::var("HEARTBEAT_URL") {
        if let Err(e) = reqwest::get(&heartbeat_url).await {
            error!("Error while calling heartbeat url: {e:?}");
        } else {
            debug!("Heartbeat url {heartbeat_url} called");
        }
    }
}

fn fetched(
    database: Database,
    disruptions: Vec<Disruption>,
    components: Components,
) -> Result<(), Box<dyn Error>> {
    let connection = database.get_connection()?;

    let mut changes = 0;
    for disruption in disruptions {
        let hash = format_hash(&disruption);
        let db_disruption = connection.query_row(
            "SELECT id, hash FROM disruption WHERE key=?",
            params![&disruption.key],
            |row| Ok((row.get::<usize, i64>(0)?, row.get::<usize, String>(1)?)),
        );
        let (send, changed, disruption_id) = match db_disruption {
            Ok((disruption_id, db_hash)) => (hash != db_hash, true, Some(disruption_id)),
            Err(_) => (true, false, None),
        };
        if send {
            changes += 1;
            // Entry changed

            let start_time_sql = disruption
                .period
                .start
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            let end_time_sql = disruption
                .period
                .end
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            connection.execute(
                "INSERT INTO disruption(key, hash, start_time, end_time) VALUES(?, ?, ?, ?)
                ON CONFLICT(key) DO UPDATE
                SET hash=excluded.hash,
                    start_time=excluded.start_time,
                    end_time=excluded.end_time",
                params![&disruption.key, hash, start_time_sql, end_time_sql],
            )?;

            components.push(
                disruption_id.unwrap_or_else(|| connection.last_insert_rowid()),
                changed,
                disruption.clone(),
            )?;
        }
    }
    debug!("{changes} disruptions found/changed");
    tokio::spawn(do_heartbeat());
    Ok(())
}

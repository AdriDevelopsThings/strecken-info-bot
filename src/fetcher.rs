use std::{env, time::Duration};

use log::{debug, error};
use strecken_info::{
    disruptions::{request_disruptions, Disruption},
    filter::DisruptionsFilter,
    revision::RevisionContext,
};
use tokio::sync::mpsc;

use crate::{
    change::{get_disruption_changes, ALL_DISRUPTION_PARTS},
    database::Database,
    error::StreckenInfoBotError,
    format::hash::format_hash,
    Components,
};

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
            revision = match revision_ctx
                .wait_for_new_revision_filtered_timeout(true, Some(Duration::from_secs(60 * 10)))
                .await
            {
                Ok(revision) => revision,
                Err(_) => {
                    revision_ctx = RevisionContext::connect()
                        .await
                        .expect("Error while trying to reconnect to websocket");
                    revision_ctx.get_first_revision().await.expect(
                        "Error while trying to reconnect to websocket and get first revision",
                    )
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(s) = rx.recv().await {
            if let Err(e) = fetched(database.clone(), s, components.clone()).await {
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

async fn fetched(
    database: Database,
    disruptions: Vec<Disruption>,
    components: Components,
) -> Result<(), StreckenInfoBotError> {
    let connection = database.get_connection().await?;

    let mut change_count = 0;
    for disruption in disruptions {
        let db_disruption = connection
            .query_opt(
                "SELECT id, hash, json FROM disruption WHERE key=$1",
                &[&disruption.key],
            )
            .await?;
        // changes: list of parts of disruption that changed
        // update: true => disruption was updated, false => disruption is new
        // disruption_id: id of disruption in database
        // contains_json: database already contains json
        let (changes, update, disruption_id, contains_json) = match db_disruption {
            Some(row) => {
                let db_disruption = row
                    .get::<_, Option<serde_json::Value>>(2)
                    .map(serde_json::from_value)
                    .transpose()?;
                let contains_json = db_disruption.is_some();
                Ok::<_, StreckenInfoBotError>((
                    get_disruption_changes(db_disruption, row.get(1), &disruption),
                    true,
                    Some(row.get(0)),
                    contains_json,
                ))
            }
            None => Ok((ALL_DISRUPTION_PARTS.to_vec(), false, None, false)), // all parts changed (new disruption)
        }?;
        if !changes.is_empty() {
            change_count += 1;
            // Entry changed

            // disruption is new or was updated, insert or update
            let returning = connection
                .query_one(
                    "INSERT INTO disruption(key, hash, start_time, end_time, json) VALUES($1, $2, $3, $4, $5)
                ON CONFLICT(key) DO UPDATE
                SET hash=EXCLUDED.hash,
                    start_time=EXCLUDED.start_time,
                    end_time=EXCLUDED.end_time,
                    json=EXCLUDED.json
                RETURNING id",
                    &[
                        &disruption.key,
                        &format_hash(&disruption),
                        &disruption.period.start,
                        &disruption.period.end,
                        &serde_json::to_value(&disruption).unwrap()
                    ],
                )
                .await?;

            components.push(
                disruption_id.unwrap_or_else(|| returning.get(0)),
                changes,
                update,
                disruption.clone(),
            );
        } else if !contains_json {
            // disruption already exists, but doesn't contain json yet
            connection
                .execute(
                    "UPDATE disruption SET json=$1 WHERE id=$2",
                    &[
                        &serde_json::to_value(&disruption)?,
                        &disruption_id.unwrap(), // disruption_id is some, never none
                    ],
                )
                .await?;
        }
    }
    debug!("{change_count} disruptions found/changed");
    tokio::spawn(do_heartbeat());
    Ok(())
}

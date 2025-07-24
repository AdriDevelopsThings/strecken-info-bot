use std::{env, time::Duration};

use log::{debug, error, info};
use strecken_info::{
    disruptions::{request_disruptions, Disruption},
    filter::{DisruptionsFilter, DisruptionsFilterTime},
    revision::RevisionContext,
};
use tokio::sync::mpsc;

use crate::{
    data::{
        strecken_info::{
            change::{get_disruption_changes, ALL_DISRUPTION_PARTS},
            StreckenInfoDisruption, STRECKEN_INFO_TYPE,
        },
        DataDisruption,
    },
    database::Database,
    error::StreckenInfoBotError,
};

pub fn start_fetching(
    database: Database,
    data_sender: mpsc::Sender<(Box<dyn DataDisruption>, bool)>,
) {
    info!("strecken-info fetching started.");
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
            let disruptions = match request_disruptions(
                DisruptionsFilter {
                    time: (DisruptionsFilterTime::Hours { hours: 24 }),
                    ..Default::default()
                },
                revision,
            )
            .await
            {
                Ok(s) => s,
                Err(e) => {
                    error!("Error while fetching disruptions: {e:?}, retrying in 10 seconds.");
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
            if let Err(e) = fetched(database.clone(), s, data_sender.clone()).await {
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
    data_sender: mpsc::Sender<(Box<dyn DataDisruption>, bool)>,
) -> Result<(), StreckenInfoBotError> {
    let connection = database.get_connection().await?;

    let mut change_count = 0;
    for disruption in disruptions {
        let db_disruption = connection
            .query_opt(
                "SELECT json FROM disruption WHERE data_source=$1 AND key=$2",
                &[&STRECKEN_INFO_TYPE, &disruption.key],
            )
            .await?;
        // changes: list of parts of disruption that changed
        // update: true => disruption was updated, false => disruption is new
        // disruption_id: id of disruption in database
        // contains_json: database already contains json
        let (changes, updated) = match db_disruption {
            Some(row) => {
                let db_disruption = row
                    .get::<_, Option<serde_json::Value>>(0)
                    .map(serde_json::from_value)
                    .transpose()?;
                Ok::<_, StreckenInfoBotError>((
                    get_disruption_changes(db_disruption, &disruption),
                    true,
                ))
            }
            None => Ok((ALL_DISRUPTION_PARTS.to_vec(), false)), // all parts changed (new disruption)
        }?;
        if !changes.is_empty() {
            change_count += 1;
            // Entry changed

            data_sender
                .send((
                    Box::new(StreckenInfoDisruption {
                        disruption: disruption.clone(),
                        changes,
                    }),
                    updated,
                ))
                .await
                .unwrap();
        }
    }
    debug!("{change_count} disruptions found/changed");
    tokio::spawn(do_heartbeat());
    Ok(())
}

use std::any::Any;

use chrono::NaiveDateTime;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{components::ComponentType, data::clone::DataDisruptionClone, Components, Database};

mod clone;
pub mod strecken_info;

pub const AVAILABLE_DATA_SOURCES: &[&str] = &["strecken_info"];

#[derive(Clone)]
pub struct DataDisruptionInformation {
    pub id: i32,
    pub disruption: Box<dyn DataDisruption>,
    pub updated: bool,
}

pub struct DatabaseRepresentation {
    pub key: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub json: Value,
}

pub trait DataDisruption: Send + Sync + DataDisruptionClone {
    fn get_type(&self) -> &'static str;
    fn format(&self, component_type: ComponentType, updated: bool) -> String;
    fn format_tws(&self) -> String;
    fn get_database_repr(&self) -> DatabaseRepresentation;
    fn as_any(&self) -> &dyn Any;
}

impl DataDisruptionInformation {
    pub fn format(&self, component_type: ComponentType) -> String {
        self.disruption.format(component_type, self.updated)
    }
}

pub async fn start_fetching(database: Database, components: Components) {
    let (tx, mut rx) = mpsc::channel::<(Box<dyn DataDisruption>, bool)>(16);

    strecken_info::start_fetching(database.clone(), tx);

    tokio::spawn(async move {
        let connection = database.get_connection().await.unwrap();
        while let Some(disruption) = rx.recv().await {
            let updated = disruption.1;
            let disruption = disruption.0;
            let database_repr = disruption.get_database_repr();
            let returning = connection
                .query_one(
                    "INSERT INTO disruption(key, data_source, start_time, end_time, json) VALUES($1, $2, $3, $4, $5)
                ON CONFLICT(data_source, key) DO UPDATE
                SET start_time=EXCLUDED.start_time,
                    end_time=EXCLUDED.end_time,
                    json=EXCLUDED.json
                RETURNING id",
                    &[
                        &database_repr.key,
                        &disruption.get_type(),
                        &database_repr.start_time,
                        &database_repr.end_time,
                        &database_repr.json,
                    ],
                )
                .await.expect("Error while inserting disruption in the database");

            let disruption = DataDisruptionInformation {
                id: returning.get(0),
                disruption,
                updated,
            };

            components.push(disruption);
        }
    });
}

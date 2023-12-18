use std::env;

use axum::{extract, routing, serve, Router};
use log::info;
use r2d2_sqlite::rusqlite::params;

use crate::Database;

async fn metrics(extract::State(database): extract::State<Database>) -> String {
    let connection = database.get_connection().unwrap();
    let metrics_prefix =
        env::var("METRICS_PREFIX").unwrap_or_else(|_| "strecken_info_telegram".to_string());
    let users: i32 = connection
        .query_row("SELECT COUNT(id) FROM user", params![], |row| row.get(0))
        .unwrap();
    let disruptions: i32 = connection.query_row("SELECT COUNT(id) FROM disruption WHERE start_time < datetime('now') AND end_time > datetime('now')", params![], |row| row.get(0)).unwrap();

    format!("{metrics_prefix}_users {users}\n{metrics_prefix}_disruptions {disruptions}")
}

pub async fn start_server(database: Database) {
    let metrics_path = env::var("METRICS_PATH").unwrap_or_else(|_| "/metrics".to_string());
    let app = Router::new()
        .route(&metrics_path, routing::get(metrics))
        .with_state(database);
    let addr = env::var("METRICS_LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Error while listening metrics server");
    info!("Metrics server is listening on {addr}");
    tokio::spawn(async move {
        serve(listener, app)
            .await
            .expect("Error while serving metrics server");
    });
}

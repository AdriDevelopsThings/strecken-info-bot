use std::env;

use axum::{extract, response::IntoResponse, routing, serve, Router};
use log::info;
use reqwest::StatusCode;

use crate::{database::DbError, Database};

impl IntoResponse for DbError {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn metrics(extract::State(database): extract::State<Database>) -> Result<String, DbError> {
    let connection = database.get_connection().await?;
    let metrics_prefix =
        env::var("METRICS_PREFIX").unwrap_or_else(|_| "strecken_info_telegram".to_string());
    let users: i64 = connection
        .query_one("SELECT COUNT(id) FROM telegram_user", &[])
        .await?
        .get(0);
    let disruptions: i64 = connection
        .query_one(
            "SELECT COUNT(id) FROM disruption WHERE start_time < NOW() AND end_time > NOW()",
            &[],
        )
        .await?
        .get(0);

    Ok(format!(
        "{metrics_prefix}_users {users}\n{metrics_prefix}_disruptions {disruptions}"
    ))
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

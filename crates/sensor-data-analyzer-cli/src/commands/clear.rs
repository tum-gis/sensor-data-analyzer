use crate::commands::ENV_VARIABLE_DATABASE_URL;
use sensor_data_analyzer::DatabaseManager;
use std::env;
use std::time::Instant;
use tracing::info;

#[tokio::main]
pub async fn run() {
    info!("Run stats");

    let database_url = env::var(ENV_VARIABLE_DATABASE_URL).unwrap();
    let maximum_of_connections = 10;
    let database_manager = DatabaseManager::new(&database_url, maximum_of_connections);

    let start = Instant::now();
    database_manager.clean().await.unwrap();
    let duration = start.elapsed();
    info!(
        "Clearing process took {:?} with {maximum_of_connections}.",
        duration
    );
}

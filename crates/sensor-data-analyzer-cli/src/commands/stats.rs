use crate::commands::ENV_VARIABLE_DATABASE_URL;
use sensor_data_analyzer::DatabaseManager;
use std::env;
use tracing::info;

#[tokio::main]
pub async fn run() {
    info!("Run stats");

    let database_url = env::var(ENV_VARIABLE_DATABASE_URL).unwrap();
    let database_manager = DatabaseManager::new(&database_url, 10);

    database_manager.run_stats().await.unwrap();
}

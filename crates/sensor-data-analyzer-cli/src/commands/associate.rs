use crate::commands::ENV_VARIABLE_DATABASE_URL;
use sensor_data_analyzer::DatabaseManager;
use std::env;
use std::time::Instant;
use tracing::info;

#[tokio::main]
pub async fn run(
    distance_threshold: f32,
    beam_intersection: bool,
    keep_temporary_table_entries: bool,
    maximum_number_connections: usize,
) {
    info!("Run associate with distance_threshold: {distance_threshold}");

    let database_url = env::var(ENV_VARIABLE_DATABASE_URL).unwrap();
    let database_manager = DatabaseManager::new(&database_url, maximum_number_connections);

    let start = Instant::now();
    database_manager
        .associate(
            distance_threshold,
            beam_intersection,
            keep_temporary_table_entries,
        )
        .await
        .unwrap();
    let duration = start.elapsed();
    info!("Association process took {:?}.", duration);
}

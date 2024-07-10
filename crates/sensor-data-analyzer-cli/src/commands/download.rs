use std::{env, fs};

use crate::commands::ENV_VARIABLE_DATABASE_URL;
use sensor_data_analyzer::DatabaseManager;
use std::path::Path;
use std::time::Instant;
use tracing::info;

#[tokio::main]
pub async fn run(
    directory_path: impl AsRef<Path>,
    keep_temporary_table_entries: bool,
    maximum_number_connections: usize,
) {
    info!("Start download");

    if directory_path.as_ref().exists() {
        fs::remove_dir_all(&directory_path).expect("TODO: panic message");
    }
    fs::create_dir_all(&directory_path).unwrap();

    let database_url = env::var(ENV_VARIABLE_DATABASE_URL).unwrap();
    let database_manager = DatabaseManager::new(&database_url, maximum_number_connections);

    let start = Instant::now();
    database_manager
        .download(&directory_path, keep_temporary_table_entries)
        .await
        .unwrap();
    let duration = start.elapsed();
    info!("Download process took {:?}.", duration);
}

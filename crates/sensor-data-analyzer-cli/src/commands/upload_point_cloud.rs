use crate::commands::ENV_VARIABLE_DATABASE_URL;
use epoint::io::LasReader;
use sensor_data_analyzer::DatabaseManager;
use std::env;
use std::path::Path;
use tracing::info;

#[tokio::main]
pub async fn run(point_cloud_file_path: impl AsRef<Path>) {
    info!("Start uploading");

    let point_cloud = LasReader::from_path(point_cloud_file_path)
        .unwrap()
        .finish()
        .unwrap()
        .0;
    info!("Loaded point cloud with {} points", point_cloud.size());

    let database_url = env::var(ENV_VARIABLE_DATABASE_URL).unwrap();
    let database_manager = DatabaseManager::new(&database_url, 10);

    database_manager.clean().await.unwrap();
    database_manager
        .upload_point_cloud(point_cloud)
        .await
        .unwrap();
}

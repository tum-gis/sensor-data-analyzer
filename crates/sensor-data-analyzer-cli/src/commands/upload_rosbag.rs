use crate::commands::ENV_VARIABLE_DATABASE_URL;

use chrono::{DateTime, Duration, Utc};
use erosbag::RosbagOpenOptions;
use sensor_data_analyzer::DatabaseManager;
use std::env;
use std::path::Path;
use std::time::Instant;
use sysinfo::System;
use tracing::{info, warn};

#[tokio::main]
pub async fn run(
    rosbag_directory_path: impl AsRef<Path>,
    ecoord_file_path: impl AsRef<Path>,
    artefact_directory_path: Option<impl AsRef<Path>>,
    step_duration: Duration,
    start_date_time: Option<DateTime<Utc>>,
    stop_date_time: Option<DateTime<Utc>>,
    start_time_offset: Option<Duration>,
    total_duration: Option<Duration>,
    maximum_number_connections: usize,
) {
    info!("Start uploading");

    let rosbag = RosbagOpenOptions::new()
        .read_write(true)
        .open(rosbag_directory_path.as_ref())
        .unwrap();
    /*if let Some(artefact_directory_path) = &artefact_directory_path {
        extract_lidar_text_mesh(&rosbag, artefact_directory_path).unwrap();
    }*/

    let reference_frames = ecoord::io::EcoordReader::from_path(ecoord_file_path)
        .unwrap()
        .finish()
        .unwrap();

    let mut sys = System::new_all();
    sys.refresh_all();
    // RAM and swap information:
    //let free_memory = (sys.total_memory() - sys.used_memory()) as f32 / (1024 * 1024 * 1024) as f32;
    //println!("free memory: {free_memory} gigabytes");
    //let maximum_of_connections: usize = (free_memory / 10.0).floor() as usize; // some estimate

    let artefact_directory_path = artefact_directory_path.map(|p| p.as_ref().to_owned());

    let rosbag_start_date_time = match rosbag.get_start_date_time() {
        Ok(Some(date_time)) => date_time,
        Ok(None) => {
            panic!("Not able to retrieve start date time from Rosbag.")
        }
        Err(error) => {
            panic!("Problem opening the file: {:?}", error);
        }
    };
    let rosbag_stop_date_time = match rosbag.get_stop_date_time() {
        Ok(Some(date_time)) => date_time,
        Ok(None) => {
            panic!("Not able to retrieve stop date time from Rosbag.")
        }
        Err(error) => {
            panic!("Problem opening the file: {:?}", error);
        }
    };
    info!(
        "Rosbag times: {rosbag_start_date_time} - {rosbag_stop_date_time} with a duration of {}",
        rosbag_stop_date_time - rosbag_start_date_time
    );

    let start_date_time: DateTime<Utc> =
        start_date_time.unwrap_or(rosbag_start_date_time) + start_time_offset.unwrap_or_default();
    let stop_date_time: DateTime<Utc> = match (total_duration, stop_date_time) {
        (Some(_total_duration), Some(stop_date_time)) => {
            warn!("Both stop_date_time and total_duration defined. Using stop_date_time");
            stop_date_time
        }
        (Some(total_duration), None) => start_date_time + total_duration,
        (None, Some(stop_date_time)) => stop_date_time,
        _ => rosbag_stop_date_time,
    };

    let start_date_time = if rosbag_start_date_time <= start_date_time {
        start_date_time
    } else {
        warn!(
            "Defined start_date_time ({}) is before rosbag's start date time ({})",
            start_date_time, rosbag_start_date_time
        );
        rosbag_start_date_time
    };
    let stop_date_time = if stop_date_time <= rosbag_stop_date_time {
        stop_date_time
    } else {
        warn!(
            "Defined stop_date_time ({}) is after rosbag's stop date time ({})",
            stop_date_time, rosbag_stop_date_time
        );
        rosbag_stop_date_time
    };

    let database_url = env::var(ENV_VARIABLE_DATABASE_URL)
        .expect("Environment variable ENV_VARIABLE_DATABASE_URL not set.");
    let database_manager = DatabaseManager::new(&database_url, maximum_number_connections);
    database_manager.clean().await.unwrap();

    let start = Instant::now();
    database_manager
        .upload_rosbag(
            rosbag,
            reference_frames,
            step_duration,
            start_date_time,
            stop_date_time,
            artefact_directory_path,
        )
        .await
        .unwrap();

    let duration = start.elapsed();
    info!(
        "Upload process took {:?} with {:?} connections.",
        duration, maximum_number_connections
    );
}

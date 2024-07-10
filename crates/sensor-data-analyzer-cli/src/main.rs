mod arguments;
mod commands;
mod util;

use std::path::{Path, PathBuf};

use crate::arguments::{Arguments, Commands};
use clap::Parser;

fn main() {
    tracing_subscriber::fmt::init();
    let arguments = Arguments::parse();

    match &arguments.command {
        Commands::Stats {} => {
            commands::stats::run();
        }
        Commands::Clear {} => {
            commands::clear::run();
        }
        Commands::UploadRosbag {
            rosbag_directory_path,
            ecoord_file_path,
            artefact_directory_path,
            start_date_time,
            stop_date_time,
            start_time_offset,
            total_duration,
            step_duration,
            maximum_number_connections,
        } => {
            let rosbag_directory_path = Path::new(rosbag_directory_path).canonicalize().unwrap();
            let ecoord_file_path = PathBuf::from(ecoord_file_path);
            let temporary_artefact_directory_path =
                artefact_directory_path.clone().map(PathBuf::from);

            commands::upload_rosbag::run(
                rosbag_directory_path,
                ecoord_file_path,
                temporary_artefact_directory_path,
                *step_duration,
                *start_date_time,
                *stop_date_time,
                *start_time_offset,
                *total_duration,
                *maximum_number_connections,
            );
        }
        Commands::UploadPointCloud {
            point_cloud_file_path,
        } => {
            let point_cloud_file_path = Path::new(point_cloud_file_path).canonicalize().unwrap();

            commands::upload_point_cloud::run(point_cloud_file_path);
        }
        Commands::Associate {
            distance_threshold,
            beam_intersection,
            keep_temporary_table_entries,
            maximum_number_connections,
        } => {
            commands::associate::run(
                *distance_threshold,
                *beam_intersection,
                *keep_temporary_table_entries,
                *maximum_number_connections,
            );
        }
        Commands::Download {
            directory_path,
            keep_temporary_table_entries,
            maximum_number_connections,
        } => {
            let directory_path = PathBuf::from(directory_path);

            commands::download::run(
                directory_path,
                *keep_temporary_table_entries,
                *maximum_number_connections,
            );
        }
    };
}

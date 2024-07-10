use crate::util::parse_duration;
use crate::util::parse_timestamp;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None, propagate_version = true)]
pub struct Arguments {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Clear database from sensor data
    Clear {},

    /// Upload ROS bag to the database
    UploadRosbag {
        /// Path to the rosbag to be uploaded
        #[clap(short, long)]
        rosbag_directory_path: String,

        /// Path to additional georeferencing
        #[clap(long)]
        ecoord_file_path: String,

        /// Path to output artefact directory
        #[clap(long)]
        artefact_directory_path: Option<String>,

        /// Start time of the upload
        /// Example: 2020-04-12 22:10:57.123456789 +02:00
        #[clap(long, value_parser = parse_timestamp)]
        start_date_time: Option<DateTime<Utc>>,

        /// Stop time of the upload
        /// Example: 2020-04-12 22:10:57.123456789 +02:00
        #[clap(long, value_parser = parse_timestamp)]
        stop_date_time: Option<DateTime<Utc>>,

        /// Duration of rosbag upload
        #[clap(long, value_parser = parse_duration)]
        start_time_offset: Option<chrono::Duration>,

        /// Duration of rosbag upload
        #[clap(long, value_parser = parse_duration)]
        total_duration: Option<chrono::Duration>,

        /// Duration of a single step
        #[clap(long, value_parser = parse_duration, default_value = "500ms")]
        step_duration: chrono::Duration,

        /// Maximum number of connections to the database
        #[clap(long, default_value = "30")]
        maximum_number_connections: usize,
    },

    /// Upload point cloud to the database
    UploadPointCloud {
        /// Path to the point cloud to be uploaded
        #[clap(short, long)]
        point_cloud_file_path: String,
    },

    /// Associate sensor data with model
    Associate {
        /// Distance between point and model threshold
        #[clap(short, long, default_value = "0.2")]
        distance_threshold: f32,

        /// Associate the points also intersecting the beams with the model surfaces
        #[clap(short, long, default_value = "false")]
        beam_intersection: bool,

        /// Keep temporary table entries
        #[clap(short, long, default_value = "false")]
        keep_temporary_table_entries: bool,

        /// Maximum number of connections to the database
        #[clap(long, default_value = "30")]
        maximum_number_connections: usize,
    },

    /// Download point clouds from the database
    Download {
        /// Directory path to the files stored
        #[clap(short, long)]
        directory_path: String,

        /// Keep temporary table entries
        #[clap(short, long, default_value = "false")]
        keep_temporary_table_entries: bool,

        /// Maximum number of connections to the database
        #[clap(long, default_value = "30")]
        maximum_number_connections: usize,
    },

    /// Stats
    Stats {},
}

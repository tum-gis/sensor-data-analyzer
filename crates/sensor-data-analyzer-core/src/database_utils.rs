use ecoord::ReferenceFrames;
use epoint::{PointCloud, PointCloudInfo, PointDataColumnType};
use erosbag::RosPointCloudColumnType;

use crate::error::Error;
use crate::models::exports::PointCloudDownloadEntry;
use itertools::{izip, Itertools};

use polars::datatypes::UInt32Chunked;
use polars::frame::DataFrame;
use polars::prelude::NamedFrom;
use polars::series::Series;
use rayon::prelude::*;

/// https://pgpointcloud.github.io/pointcloud/concepts/binary.html#dimensional
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
#[repr(u32)]
pub enum DimensionCompression {
    NoCompression = 0,
    RunLengthCompression = 1,
    SignificantBitsRemoval = 2,
    Deflate = 3,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
#[repr(u8)]
pub enum Endianess {
    Xdr = 0,
    Ndr = 1,
}

struct PathUncompressedBinary {
    endianness: u8,
    pcid: u32,
    compression: u32,
    npoints: u32,
    pointdata: Vec<i64>,
}

pub fn insert_point_cloud(point_cloud: &epoint::PointCloud) -> Result<Vec<String>, Error> {
    let x_values = point_cloud.point_data.get_x_values();
    let y_values = point_cloud.point_data.get_y_values();
    let z_values = point_cloud.point_data.get_z_values();

    let id_values = point_cloud.point_data.get_id_values()?;
    let timestamp_seconds_values = point_cloud.point_data.get_timestamp_sec_values()?;
    let timestamp_nano_seconds_values = point_cloud.point_data.get_timestamp_nanosec_values()?;
    let intensity_values = point_cloud.point_data.get_intensity_values()?;

    let beam_origin_x_values = point_cloud.point_data.get_beam_origin_x_values()?;
    let beam_origin_y_values = point_cloud.point_data.get_beam_origin_y_values()?;
    let beam_origin_z_values = point_cloud.point_data.get_beam_origin_z_values()?;

    let ros_message_id_values: &UInt32Chunked = point_cloud
        .point_data()
        .data_frame
        .column(RosPointCloudColumnType::RosMessageId.as_str())
        .unwrap()
        .u32()
        .unwrap();

    let ros_point_id_values: &UInt32Chunked = point_cloud
        .point_data()
        .data_frame
        .column(RosPointCloudColumnType::RosPointId.as_str())
        .unwrap()
        .u32()
        .unwrap();
    // let feature_id_values = vec![-1; point_cloud.point_data().height()];

    let individual_entries: Vec<String> = izip!(
        x_values,
        y_values,
        z_values,
        id_values,
        timestamp_seconds_values,
        timestamp_nano_seconds_values,
        intensity_values,
        beam_origin_x_values,
        beam_origin_y_values,
        beam_origin_z_values,
        ros_message_id_values,
        ros_point_id_values,
    )
    .into_iter()
    .map(
        |(
            x,
            y,
            z,
            id,
            timestamp_sec,
            timestamp_nanosec,
            intensity,
            beam_origin_x,
            beam_origin_y,
            beam_origin_z,
            ros_message_id,
            ros_point_id,
        )| {
            format!(
                "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
                x.unwrap(),
                y.unwrap(),
                z.unwrap(),
                id.unwrap(),
                timestamp_sec.unwrap(),
                timestamp_nanosec.unwrap(),
                intensity.unwrap(),
                beam_origin_x.unwrap(),
                beam_origin_y.unwrap(),
                beam_origin_z.unwrap(),
                ros_message_id.unwrap(),
                ros_point_id.unwrap(),
            )
        },
    )
    .collect();

    let merged: String = individual_entries
        .into_iter()
        .intersperse(", ".into())
        .collect();

    let query = format!(
        "INSERT INTO sensor_data.point_cloud_upload (pa)
    SELECT PC_MakePatch(1, ARRAY[{merged}]);"
    );
    Ok(vec![query])
}

pub fn derive_point_cloud(
    database_point_cloud: Vec<PointCloudDownloadEntry>,
) -> Result<epoint::PointCloud, Error> {
    let x_series = Series::new(
        PointDataColumnType::X.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.x)
            .collect::<Vec<f64>>(),
    );
    let y_series = Series::new(
        PointDataColumnType::Y.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.y)
            .collect::<Vec<f64>>(),
    );
    let z_series = Series::new(
        PointDataColumnType::Z.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.z)
            .collect::<Vec<f64>>(),
    );
    let id_series = Series::new(
        PointDataColumnType::Id.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.point_id as u64)
            .collect::<Vec<u64>>(),
    );
    let timestamp_seconds_series = Series::new(
        PointDataColumnType::TimestampSeconds.as_str(),
        database_point_cloud
            .iter()
            .map(|t| t.timestamp_sec as i64)
            .collect::<Vec<i64>>(),
    );
    let timestamp_nanoseconds_series = Series::new(
        PointDataColumnType::TimestampNanoSeconds.as_str(),
        database_point_cloud
            .iter()
            .map(|t| t.timestamp_nanosec as u32)
            .collect::<Vec<u32>>(),
    );
    let intensity_series = Series::new(
        PointDataColumnType::Intensity.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.intensity as f32)
            .collect::<Vec<f32>>(),
    );
    let beam_origin_x_series = Series::new(
        PointDataColumnType::BeamOriginX.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.x)
            .collect::<Vec<f64>>(),
    );
    let beam_origin_y_series = Series::new(
        PointDataColumnType::BeamOriginY.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.y)
            .collect::<Vec<f64>>(),
    );
    let beam_origin_z_series = Series::new(
        PointDataColumnType::BeamOriginZ.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.z)
            .collect::<Vec<f64>>(),
    );
    let beam_length_series = Series::new(
        "beam_length",
        database_point_cloud
            .iter()
            .map(|p| p.beam_length)
            .collect::<Vec<f64>>(),
    );
    let ros_point_id_series = Series::new(
        RosPointCloudColumnType::RosPointId.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.ros_point_id)
            .collect::<Vec<i32>>(),
    );
    let ros_message_id_series = Series::new(
        RosPointCloudColumnType::RosMessageId.as_str(),
        database_point_cloud
            .iter()
            .map(|p| p.ros_message_id)
            .collect::<Vec<i32>>(),
    );

    let gml_id_series = Series::new(
        "gml_id",
        database_point_cloud
            .iter()
            .map(|t| t.gml_id.clone().unwrap_or("".to_string()))
            .collect::<Vec<String>>(),
    );
    let gml_name_series = Series::new(
        "gml_name",
        database_point_cloud
            .iter()
            .map(|t| t.gml_name.clone().unwrap_or("".to_string()))
            .collect::<Vec<String>>(),
    );
    let classname_series = Series::new(
        "classname",
        database_point_cloud
            .iter()
            .map(|t| t.classname.clone().unwrap_or("".to_string()))
            .collect::<Vec<String>>(),
    );
    let surface_distance_series = Series::new(
        "surface_distance",
        database_point_cloud
            .iter()
            .map(|p| p.surface_distance.map(|x| x as f32).unwrap_or(f32::NAN))
            .collect::<Vec<f32>>(),
    );
    let intersection_angle_series = Series::new(
        "intersection_angle",
        database_point_cloud
            .iter()
            .map(|p| p.intersection_angle.map(|x| x as f32).unwrap_or(f32::NAN))
            .collect::<Vec<f32>>(),
    );

    let columns = vec![
        x_series,
        y_series,
        z_series,
        id_series,
        timestamp_seconds_series,
        timestamp_nanoseconds_series,
        intensity_series,
        beam_origin_x_series,
        beam_origin_y_series,
        beam_origin_z_series,
        beam_length_series,
        ros_point_id_series,
        ros_message_id_series,
        gml_id_series,
        gml_name_series,
        classname_series,
        surface_distance_series,
        intersection_angle_series,
    ];
    let df = DataFrame::new(columns).unwrap();
    let point_cloud_info = PointCloudInfo::new(None);
    let point_cloud =
        PointCloud::from_data_frame(df, point_cloud_info, ReferenceFrames::default()).unwrap();

    Ok(point_cloud)
}

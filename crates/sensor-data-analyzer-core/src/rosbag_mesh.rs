use crate::error::Error;
use crate::sphere::{
    SphericalRasterizationAxis, SphericalRasterizationTransform, UnitSphericalCellIndex3,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use diesel_async::RunQueryDsl;
use ecoord::FrameId;
use emesh::Polygon;
use epoint::PointDataColumnType;
use erosbag::Rosbag;
use nalgebra::Point3;
use polars::prelude::*;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use tracing::info;

pub fn extract_lidar_text_mesh(
    rosbag: &Rosbag,
    artefact_directory_path: impl AsRef<Path>,
) -> Result<(), Error> {
    let start_time: Option<DateTime<Utc>> = Some(Utc.timestamp_opt(1605702866, 0).unwrap());
    let stop_time: Option<DateTime<Utc>> = start_time.map(|x| x + Duration::milliseconds(100));

    let mut complete_point_cloud = rosbag.get_point_clouds(&start_time, &stop_time)?;
    let distinct_frame_ids = complete_point_cloud.get_distinct_frame_ids();
    info!("frame ids: {:?}", distinct_frame_ids);
    complete_point_cloud
        .derive_spherical_points()
        .expect("TODO: panic message");

    info!("Write complete point cloud");
    let mut complete_resolved_point_cloud = complete_point_cloud.clone();
    complete_resolved_point_cloud
        .resolve_to_frame(FrameId::from("base_link"))
        .unwrap();
    let p = artefact_directory_path
        .as_ref()
        .join(PathBuf::from("complete_base_link.xyz"));
    epoint::io::XyzWriter::new(p).finish(&complete_resolved_point_cloud)?;

    info!("Write individual sensor point cloud");
    let point_cloud_front =
        complete_point_cloud.filter_by_frame_id(&FrameId::from("lidar_front_center"))?;
    let mesh = point_cloud_to_mesh(&point_cloud_front, artefact_directory_path.as_ref())?;
    let graphics_mesh = emesh_converter::mesh_to_graphics(mesh)?;
    let p = artefact_directory_path
        .as_ref()
        .join(PathBuf::from("mesh.gltf"));
    egraphics::io::EgraphicsExporter::new(p)
        .with_derive_obj_file(true)
        .finish(graphics_mesh)?;

    let artefact_directory_path = artefact_directory_path.as_ref();
    std::fs::create_dir_all(artefact_directory_path)?;
    let p = artefact_directory_path.join(PathBuf::from("lidar_front_left.xyz"));
    epoint::io::XyzWriter::new(p).finish(&point_cloud_front)?;

    Ok(())
}

fn point_cloud_to_mesh(
    point_cloud: &epoint::PointCloud,
    artefact_directory_path: impl AsRef<Path>,
) -> Result<emesh::Mesh, Error> {
    let mut spherical_point_cloud = point_cloud.clone();
    spherical_point_cloud.derive_spherical_points()?;

    // https://velodynelidar.com/wp-content/uploads/2019/12/63-9243-Rev-E-VLP-16-User-Manual.pdf
    let config = SphericalRasterizationTransform::new(
        SphericalRasterizationAxis::from_deg(-180.0, 180.0, 0.1990656, 0.0),
        SphericalRasterizationAxis::from_deg(-15.0, 15.0, 1.875, 1.875 / 2.0),
    );
    rasterize_point_cloud(&mut spherical_point_cloud, &config);
    calculate_cell_center_distance(&mut spherical_point_cloud, &config)?;
    remove_cell_duplicates(&mut spherical_point_cloud)?;

    let artefact_directory_path = artefact_directory_path.as_ref();
    std::fs::create_dir_all(artefact_directory_path)?;
    let p = artefact_directory_path.join(PathBuf::from("lidar_front_left_raster.xyz"));
    epoint::io::XyzWriter::new(p).finish(&spherical_point_cloud)?;

    let mesh = generate_mesh_from_spherical_point_cloud(&spherical_point_cloud)?;
    Ok(mesh)
}

const COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR: &str = "spherical_elevation_index";
const COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR: &str = "spherical_azimuth_index";

pub fn rasterize_point_cloud(
    point_cloud: &mut epoint::PointCloud,
    config: &SphericalRasterizationTransform,
) {
    let spherical_elevation_index_values: Vec<i32> = point_cloud
        .point_data
        .get_spherical_elevation_values()
        .unwrap()
        .into_iter()
        .map(|e| config.elevation().transform_to_grid_cell_index(e.unwrap()))
        .collect();
    let spherical_elevation_index_series = Series::new(
        COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR,
        spherical_elevation_index_values,
    );
    point_cloud
        .point_data
        .data_frame
        .with_column(spherical_elevation_index_series)
        .unwrap();

    let spherical_azimuth_index_values: Vec<i32> = point_cloud
        .point_data
        .get_spherical_azimuth_values()
        .unwrap()
        .into_iter()
        .map(|a| config.azimuth().transform_to_grid_cell_index(a.unwrap()))
        .collect();
    let spherical_azimuth_index_series = Series::new(
        COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR,
        spherical_azimuth_index_values,
    );
    point_cloud
        .point_data
        .data_frame
        .with_column(spherical_azimuth_index_series)
        .unwrap();
}

const COLUMN_NAME_CELL_CENTER_DISTANCE_STR: &str = "cell_center_distance";

pub fn calculate_cell_center_distance(
    point_cloud: &mut epoint::PointCloud,
    transformer: &SphericalRasterizationTransform,
) -> Result<(), Error> {
    let spherical_points = point_cloud.point_data.get_all_spherical_points()?;
    let spherical_elevation_index_values = point_cloud
        .point_data
        .data_frame
        .column(COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR)?
        .i32()
        .expect("type must be i32");
    let spherical_azimuth_index_values = point_cloud
        .point_data
        .data_frame
        .column(COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR)?
        .i32()
        .expect("type must be i32");
    let distance_values: Vec<f64> = (0..point_cloud.point_data.data_frame.height())
        .into_par_iter()
        .map(|i: usize| {
            transformer
                .transform_to_point(UnitSphericalCellIndex3::new(
                    spherical_azimuth_index_values.get(i).unwrap(),
                    spherical_elevation_index_values.get(i).unwrap(),
                ))
                .rad_distance((*spherical_points.get(i).expect("s")).into())
        })
        .collect();
    let distance_series = Series::new(COLUMN_NAME_CELL_CENTER_DISTANCE_STR, distance_values);

    point_cloud
        .point_data
        .data_frame
        .with_column(distance_series)
        .unwrap();

    Ok(())
}

pub fn remove_cell_duplicates(point_cloud: &mut epoint::PointCloud) -> Result<(), Error> {
    let point_data_distinct_cells = point_cloud
        .point_data
        .data_frame
        .clone()
        .lazy()
        .group_by([
            col(COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR),
            col(COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR),
        ])
        .agg(&[all()
            .sort_by(
                [col(COLUMN_NAME_CELL_CENTER_DISTANCE_STR)],
                SortMultipleOptions::default(),
            )
            .first()])
        .collect()?;

    point_cloud.point_data.data_frame = point_data_distinct_cells;
    Ok(())
}

pub fn generate_mesh_from_spherical_point_cloud(
    point_cloud: &epoint::PointCloud,
) -> Result<emesh::Mesh, Error> {
    let mut mesh = emesh::Mesh::new(vec![], vec![]);

    /*let spherical_elevation_index_min = point_cloud
        .point_data
        .data_frame
        .column(COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR)?
        .i32()?
        .min()
        .expect("type must be i32");
    let spherical_elevation_index_max = point_cloud
        .point_data
        .data_frame
        .column(COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR)?
        .i32()?
        .max()
        .expect("type must be i32");

    let spherical_azimuth_index_min = point_cloud
        .point_data
        .data_frame
        .column(COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR)?
        .i32()?
        .min()
        .expect("type must be i32");
    let spherical_azimuth_index_max = point_cloud
        .point_data
        .data_frame
        .column(COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR)?
        .i32()?
        .max()
        .expect("type must be i32");*/

    let sorted_df = point_cloud
        .point_data
        .data_frame
        /*.select([
            COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR,
            COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR,
        ])?*/
        .sort(
            [
                COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR,
                COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR,
            ],
            SortMultipleOptions::default(),
        )?;
    println!("{:?}", sorted_df);

    for current_index in 0..sorted_df.height() {
        let current_row = sorted_df.slice(current_index as i64, 1);
        let current_spherical_cell_index = get_spherical_cell_index(&current_row)?;
        let current_point = get_point(&current_row)?;

        let right_spherical_cell_index = UnitSphericalCellIndex3::new(
            current_spherical_cell_index.azimuth() + 1,
            current_spherical_cell_index.elevation(),
        );
        let right_point = get_row_point(&right_spherical_cell_index, &sorted_df);

        let upper_spherical_cell_index = UnitSphericalCellIndex3::new(
            current_spherical_cell_index.azimuth(),
            current_spherical_cell_index.elevation() + 1,
        );
        let upper_point = get_row_point(&upper_spherical_cell_index, &sorted_df);

        let upper_right_spherical_cell_index = UnitSphericalCellIndex3::new(
            current_spherical_cell_index.azimuth() + 1,
            current_spherical_cell_index.elevation() + 1,
        );
        let upper_right_point = get_row_point(&upper_right_spherical_cell_index, &sorted_df);

        if right_point.is_some() && upper_point.is_some() && upper_right_point.is_some() {
            let polygon = Polygon::new(vec![
                current_point,
                upper_point.unwrap(),
                right_point.unwrap(),
            ])
            .unwrap();
            mesh.add_polygon(&polygon, None);

            let polygon = Polygon::new(vec![
                right_point.unwrap(),
                upper_point.unwrap(),
                upper_right_point.unwrap(),
            ])
            .unwrap();
            mesh.add_polygon(&polygon, None);
        }
    }

    Ok(mesh)
}

fn get_spherical_cell_index(row: &DataFrame) -> Result<UnitSphericalCellIndex3, Error> {
    let spherical_elevation_index = row
        .column(COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR)?
        .i32()?
        .get(0)
        .expect("type must be i32");
    let spherical_azimuth_index = row
        .column(COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR)?
        .i32()?
        .get(0)
        .expect("type must be i32");

    let unit_spherical_cell_index =
        UnitSphericalCellIndex3::new(spherical_azimuth_index, spherical_elevation_index);
    Ok(unit_spherical_cell_index)
}

fn get_row_point(
    unit_spherical_cell_index: &UnitSphericalCellIndex3,
    df: &DataFrame,
) -> Option<Point3<f64>> {
    let row = df
        .clone()
        .lazy()
        .filter(
            col(COLUMN_NAME_SPHERICAL_ELEVATION_INDEX_STR)
                .eq(lit(unit_spherical_cell_index.elevation()))
                .and(
                    col(COLUMN_NAME_SPHERICAL_AZIMUTH_INDEX_STR)
                        .eq(unit_spherical_cell_index.azimuth()),
                ),
        )
        .collect()
        .unwrap();

    if row.height() == 0 {
        None
    } else {
        Some(get_point(&row).unwrap())
    }
}

fn get_point(row: &DataFrame) -> Result<Point3<f64>, Error> {
    let x_value = row
        .column(PointDataColumnType::X.as_str())?
        .f64()?
        .get(0)
        .expect("type must be f64");
    let y_value = row
        .column(PointDataColumnType::Y.as_str())?
        .f64()?
        .get(0)
        .expect("type must be f64");
    let z_value = row
        .column(PointDataColumnType::Z.as_str())?
        .f64()?
        .get(0)
        .expect("type must be f64");

    let point = Point3::new(x_value, y_value, z_value);
    Ok(point)
}

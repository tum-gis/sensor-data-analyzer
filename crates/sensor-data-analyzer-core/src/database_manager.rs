use crate::database_utils::{derive_point_cloud, insert_point_cloud};
use crate::diesel::ExpressionMethods;
use crate::error::Error;
use crate::models::exports::PointCloudDownloadEntry;
use crate::schema;
use crate::schema::sensor_data::beam::patch_id;
use chrono::Duration as ChronoDuration;
use chrono::{DateTime, Utc};
use diesel::QueryDsl;
use diesel_async::pooled_connection::deadpool::Object;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use epoint::io::{EpointWriter, XyzWriter};
use epoint::transform::deterministic_downsample;
use epoint::{PointCloud, PointDataColumnType};
use erosbag::RosPointCloudColumnType;
use rayon::prelude::*;
use std::fs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tokio::task::JoinHandle;
use tracing::info;

/// Manages the database
pub struct DatabaseManager {
    pub(crate) connection_pool: Pool<AsyncPgConnection>,
}

impl DatabaseManager {
    pub fn new(database_url: &str, maximum_number_connections: usize) -> Self {
        info!("Number of connections: {maximum_number_connections}");

        // create a new connection pool with the default config
        let config: AsyncDieselConnectionManager<AsyncPgConnection> =
            AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
        let mut builder = Pool::builder(config).max_size(maximum_number_connections);
        let connection_pool = builder.build().unwrap();

        Self { connection_pool }
    }

    pub async fn clean(&self) -> Result<(), Error> {
        let mut connection: Object<AsyncPgConnection> = self.connection_pool.get().await.unwrap();

        self.clean_download_tables().await?;
        self.clean_association_tables().await?;

        info!("Deleting entries in table point_cloud_upload");
        let query = "TRUNCATE TABLE \
        sensor_data.point_cloud_upload CASCADE;"
            .to_string();
        diesel::sql_query(query).execute(&mut connection).await?;

        Ok(())
    }

    pub async fn clean_association_tables(&self) -> Result<(), Error> {
        let mut connection = self.connection_pool.get().await.unwrap();

        info!(
            "Deleting entries in table feature_geometry_data, association_beam_model, association_point_model, beam"
        );

        let query = "TRUNCATE TABLE \
                sensor_data.feature_geometry_data,\
                sensor_data.association_beam_model,\
                sensor_data.association_point_model,\
                sensor_data.beam CASCADE;"
            .to_string();
        diesel::sql_query(query).execute(&mut connection).await?;

        Ok(())
    }

    pub async fn clean_download_tables(&self) -> Result<(), Error> {
        let mut connection = self.connection_pool.get().await.unwrap();

        info!("Deleting entries in table point_cloud_download");
        let query = "TRUNCATE TABLE \
        sensor_data.point_cloud_download  CASCADE;"
            .to_string();
        diesel::sql_query(query).execute(&mut connection).await?;

        Ok(())
    }

    pub async fn run_stats(&self) -> Result<(), Error> {
        let mut connection = self.connection_pool.get().await.unwrap();

        //let res: Vec<models::sensor_data::Patches> = schema::sensor_data::patches::dsl::patches
        //    .load::<models::sensor_data::Patches>(&mut connection)
        //    .expect("Error loading messages");

        let res: Vec<i32> = schema::sensor_data::point_cloud_upload::dsl::point_cloud_upload
            .select(schema::sensor_data::point_cloud_upload::id)
            .load(&mut connection)
            .await?;

        info!("Length: {}", res.len());
        Ok(())
    }

    pub async fn upload_rosbag(
        &self,
        rosbag: erosbag::Rosbag,
        reference_frames: ecoord::ReferenceFrames,
        step_duration: ChronoDuration,
        start_date_time: DateTime<Utc>,
        stop_date_time: DateTime<Utc>,
        artefact_directory_path: Option<PathBuf>,
    ) -> Result<(), Error> {
        let total_duration = stop_date_time - start_date_time;
        //let stop_time: DateTime<Utc> = rosbag.get_stop_date_time()?.unwrap();
        let total_steps: i32 =
            (total_duration.num_milliseconds() / step_duration.num_milliseconds()) as i32;
        info!(
            "rosbag duration: {} ({} - {})",
            total_duration, start_date_time, stop_date_time
        );
        //rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();
        let number_of_steps = total_steps;

        // check: https://docs.rs/diesel-async/0.2.0/diesel_async/
        // into_par_iter()
        let point_clouds: Vec<PointCloud> = (0..number_of_steps)
            .map(|step| {
                info!("Extracting point clouds: {}/{}", step, number_of_steps);
                let step_start_time = start_date_time + step_duration * step;
                let step_stop_time = step_start_time + step_duration;

                rosbag
                    .get_point_clouds(&Some(step_start_time), &Some(step_stop_time))
                    .unwrap()
            })
            .collect();

        if let Some(artefact_directory_path) = &artefact_directory_path {
            if artefact_directory_path.exists() {
                fs::remove_dir_all(&artefact_directory_path)?;
            }
            create_dir_all(&artefact_directory_path)?;
        }

        let database_point_cloud_base_path: Option<PathBuf> = artefact_directory_path
            .clone()
            .map(|p| p.join("test_point_cloud_for_db"));
        if let Some(p) = &database_point_cloud_base_path {
            create_dir_all(p)?;
        }
        let database_point_cloud_xyz_base_path: Option<PathBuf> = artefact_directory_path
            .clone()
            .map(|p| p.join("test_point_cloud_for_db_xyz"));
        if let Some(p) = &database_point_cloud_xyz_base_path {
            create_dir_all(p)?;
        }

        info!("Georeferencing point clouds");
        let georeferenced_point_clouds: Vec<PointCloud> = point_clouds
            .into_par_iter()
            .enumerate()
            .map(|(step, mut point_cloud)| {
                // let p = point_cloud.reference_frames().
                // let stop_time: DateTime<Utc> = Utc.timestamp_opt(1579007201, 173003500).unwrap();
                //let c = point_cloud.reference_frames().get_channel_ids();
                //dbg!("c: {}", c);

                // point_cloud.resolve_to_frame("slam_map".into()).unwrap();

                let merged_reference_frames = ecoord::merge(&[
                    point_cloud.reference_frames().clone(),
                    reference_frames.clone(),
                ])
                .unwrap();

                point_cloud.set_reference_frames(merged_reference_frames);
                point_cloud
                    .point_data
                    .add_sequential_id()
                    .expect("should work");
                point_cloud
                    .resolve_to_frame("world".into())
                    .expect("resolving should work");

                if let Some(database_point_cloud_base_path) = database_point_cloud_base_path.clone()
                {
                    let p =
                        database_point_cloud_base_path.join(PathBuf::from(format!("{step}.tar")));
                    EpointWriter::from_path(p)
                        .unwrap()
                        .with_compressed(false)
                        .finish(point_cloud.clone())
                        .expect("Writing should work");
                }

                if let Some(database_point_cloud_xyz_base_path) =
                    database_point_cloud_xyz_base_path.clone()
                {
                    let downsampled_point_cloud =
                        deterministic_downsample(&point_cloud, 100000, Some(123)).unwrap();

                    let p = database_point_cloud_xyz_base_path
                        .join(PathBuf::from(format!("{step}.xyz")));
                    XyzWriter::new(p)
                        //.with_frame_id("slam_map".into())
                        .finish(&downsampled_point_cloud)
                        .expect("Writing should work");
                }

                point_cloud
            })
            .collect();

        info!("Start uploading");
        let mut handles: Vec<JoinHandle<()>> = vec![];

        for current_point_cloud in georeferenced_point_clouds {
            let connection = self.connection_pool.get().await.unwrap();

            let current_handle = tokio::spawn(async move {
                upload_point_cloud_direct(connection, &current_point_cloud)
                    .await
                    .unwrap();
            });
            handles.push(current_handle);
        }

        for current_handle in handles {
            current_handle.await.unwrap();
        }

        /*point_clouds.iter().for_each(|c| {
            self.upload_point_cloud(c).await?;
        });*/
        info!("Finished uploading");
        Ok(())
    }

    pub async fn upload_point_cloud(&self, mut point_cloud: PointCloud) -> Result<(), Error> {
        point_cloud
            .point_data
            .add_sequential_id()
            .expect("should work");

        point_cloud.point_data.add_i64_column(
            PointDataColumnType::TimestampSeconds.as_str(),
            vec![0i64; point_cloud.size()],
        )?;
        point_cloud.point_data.add_u32_column(
            PointDataColumnType::TimestampNanoSeconds.as_str(),
            vec![0u32; point_cloud.size()],
        )?;
        point_cloud.point_data.add_f32_column(
            PointDataColumnType::Intensity.as_str(),
            vec![0f32; point_cloud.size()],
        )?;
        point_cloud.point_data.add_f64_column(
            PointDataColumnType::BeamOriginX.as_str(),
            vec![0f64; point_cloud.size()],
        )?;
        point_cloud.point_data.add_f64_column(
            PointDataColumnType::BeamOriginY.as_str(),
            vec![0f64; point_cloud.size()],
        )?;
        point_cloud.point_data.add_f64_column(
            PointDataColumnType::BeamOriginZ.as_str(),
            vec![0f64; point_cloud.size()],
        )?;
        point_cloud.point_data.add_u32_column(
            RosPointCloudColumnType::RosMessageId.as_str(),
            vec![0u32; point_cloud.size()],
        )?;
        point_cloud.point_data.add_u32_column(
            RosPointCloudColumnType::RosPointId.as_str(),
            vec![0u32; point_cloud.size()],
        )?;

        let id_min = point_cloud.point_data.get_id_min()?.expect("must be there");
        let id_max = point_cloud.point_data.get_id_max()?.expect("must be there");
        let step_size = 100000;
        info!(
            "Start uploading {} steps",
            (id_max - id_min) / step_size as u64
        );

        let mut handles: Vec<JoinHandle<()>> = vec![];

        for current_id in (id_min..id_max).step_by(step_size) {
            let current_id_max = current_id + step_size as u64 - 1;
            let current_point_cloud =
                point_cloud.filter_by_id_range(Some(current_id), Some(current_id_max))?;

            let connection: Object<AsyncPgConnection> = self.connection_pool.get().await.unwrap();

            info!(
                "Uploading point cloud with {} points in the ID range: {}-{}",
                current_point_cloud.size(),
                current_id,
                current_id_max
            );
            let current_handle = tokio::spawn(async move {
                upload_point_cloud_direct(connection, &current_point_cloud)
                    .await
                    .unwrap();
            });
            handles.push(current_handle);
        }

        for current_handle in handles {
            current_handle.await.unwrap();
        }

        info!("Finished uploading");
        Ok(())
    }

    pub async fn associate(
        &self,
        distance_threshold: f32,
        beam_intersection: bool,
        keep_temporary_table_entries: bool,
    ) -> Result<(), Error> {
        self.clean_association_tables().await?;

        //let connection: Object<AsyncDieselConnectionManager<AsyncPgConnection>> =
        //   self.connection_pool.get().await.unwrap();
        //drop_association_index(connection).await?;

        if beam_intersection {
            info!("Explode feature geometry data");
            let connection: Object<AsyncPgConnection> = self.connection_pool.get().await.unwrap();
            explode_feature_geometry_data(connection).await?;
        }

        let mut connection: Object<AsyncPgConnection> = self.connection_pool.get().await.unwrap();
        let patch_ids: Vec<i32> = schema::sensor_data::point_cloud_upload::dsl::point_cloud_upload
            .select(schema::sensor_data::point_cloud_upload::id)
            .load(&mut connection)
            .await?;
        //dbg!("{}", id);

        let mut handles: Vec<JoinHandle<()>> = vec![];
        for current_patch_id in patch_ids.into_iter() {
            let connection: Object<AsyncPgConnection> = self.connection_pool.get().await.unwrap();

            let current_handle = tokio::spawn(async move {
                associate_points(
                    connection,
                    current_patch_id,
                    distance_threshold,
                    beam_intersection,
                    keep_temporary_table_entries,
                )
                .await
                .unwrap();
            });
            handles.push(current_handle);
        }

        for current_handle in handles {
            current_handle.await.unwrap();
        }

        //let connection: Object<AsyncDieselConnectionManager<AsyncPgConnection>> =
        //    self.connection_pool.get().await.unwrap();
        //create_association_index(connection).await?;

        Ok(())
    }

    pub async fn download(
        &self,
        directory_path: impl AsRef<Path>,
        keep_temporary_table_entries: bool,
    ) -> Result<(), Error> {
        self.clean_download_tables().await?;

        let mut connection: Object<AsyncPgConnection> = self.connection_pool.get().await.unwrap();
        let patch_ids: Vec<i32> = schema::sensor_data::point_cloud_upload::dsl::point_cloud_upload
            .select(schema::sensor_data::point_cloud_upload::id)
            .load(&mut connection)
            .await?;

        let mut handles: Vec<JoinHandle<()>> = vec![];
        for current_patch_id in patch_ids.into_iter() {
            let connection: Object<AsyncPgConnection> = self.connection_pool.get().await.unwrap();
            let path = directory_path
                .as_ref()
                .to_owned()
                .clone()
                .join(current_patch_id.to_string() + ".xyz");

            let current_handle = tokio::spawn(async move {
                let point_cloud = download_associate_points(
                    connection,
                    current_patch_id,
                    keep_temporary_table_entries,
                )
                .await
                .unwrap();
                let colorized_point_cloud =
                    epoint::transform::colorize::colorize_by_column_hash(&point_cloud, "gml_id")
                        .unwrap();
                XyzWriter::new(path)
                    //.with_compressed(false)
                    .finish(&colorized_point_cloud)
                    .unwrap();
            });
            handles.push(current_handle);
        }

        for current_handle in handles {
            current_handle.await.unwrap();
        }

        Ok(())
    }
}

async fn upload_point_cloud_direct(
    mut connection: Object<AsyncPgConnection>,
    point_cloud: &epoint::PointCloud,
) -> Result<(), Error> {
    let queries = insert_point_cloud(point_cloud)?;
    for query in queries {
        // fs::write("./query.txt", &query).expect("Unable to write file");
        diesel::sql_query(&query).execute(&mut connection).await?;
    }

    info!("Uploaded number of points: {}", point_cloud.size());
    Ok(())
}

/*async fn create_association_index(
    mut connection: Object<AsyncDieselConnectionManager<AsyncPgConnection>>,
) -> Result<(), Error> {
    info!("Create index for association");
    let query = format!("CREATE INDEX point_cloud_upload_exploded_geometry_index ON sensor_data.point_cloud_upload_exploded USING gist(geometry gist_geometry_ops_nd);");
    diesel::sql_query(&query).execute(&mut connection).await?;

    Ok(())
}

async fn drop_association_index(
    mut connection: Object<AsyncDieselConnectionManager<AsyncPgConnection>>,
) -> Result<(), Error> {
    info!("Drop index for association");

    let query = format!("DROP INDEX IF EXISTS sensor_data.association_point_model_point_index;");
    diesel::sql_query(&query).execute(&mut connection).await?;

    Ok(())
}*/

async fn associate_points(
    mut connection: Object<AsyncPgConnection>,
    current_patch_id: i32,
    distance_threshold: f32,
    beam_intersection: bool,
    keep_temporary_table_entries: bool,
) -> Result<(), Error> {
    info!("Exploding patch with id: {current_patch_id}");
    let reflection_line_length = distance_threshold * 2.0;

    let query = format!("INSERT INTO sensor_data.beam (patch_id, point_id, timestamp_sec, timestamp_nanosec, intensity, origin, reflection, line, length, reflection_line, ros_message_id, ros_point_id)
SELECT
    patch_id,
    point_id,
    timestamp_sec,
    timestamp_nanosec,
    intensity,
    origin,
    reflection,
    line,
    ST_3DLength(line),
    ST_Translate(
           ST_Scale(
                   ST_Translate(line, -ST_X(midpoint), -ST_Y(midpoint), -ST_Z(midpoint)),
                   {reflection_line_length}/length, {reflection_line_length}/length, {reflection_line_length}/length),
           ST_X(reflection), ST_Y(reflection), ST_Z(reflection)) as reflection_line,
    ros_message_id,
    ros_point_id
FROM
    (SELECT
         ST_MakeLine(origin, reflection) as line,
         ST_3DDistance(origin, reflection) as length,
         ST_LineInterpolatePoint(ST_MakeLine(origin, reflection), 0.5) AS midpoint,
         *
    FROM (SELECT id as patch_id,
           PC_Get(pc_explode(pa), 'id') as point_id,
           PC_Get(pc_explode(pa), 'timestamp_sec') as timestamp_sec,
           PC_Get(pc_explode(pa), 'timestamp_nanosec') as timestamp_nanosec,
           PC_Get(pc_explode(pa), 'intensity') as intensity,
           ST_SetSRID(st_makepoint(PC_Get(PC_Explode(pa), 'beam_origin_x'), PC_Get(PC_Explode(pa), 'beam_origin_y'), PC_Get(PC_Explode(pa), 'beam_origin_z')), ST_SRID(pc_explode(pa)::geometry)) as origin,
           PC_Explode(pa)::geometry as reflection,
           PC_Get(pc_explode(pa), 'ros_message_id') as ros_message_id,
           PC_Get(pc_explode(pa), 'ros_point_id') as ros_point_id
    FROM sensor_data.point_cloud_upload
    WHERE point_cloud_upload.id = {current_patch_id}) as source_point_exploded) as pc;");
    diesel::sql_query(query).execute(&mut connection).await?;

    info!("Associating point-model with patch_id: {current_patch_id}");
    let query = format!(
        "INSERT INTO sensor_data.association_point_model (beam_id, feature_id, distance)
    SELECT DISTINCT beam.id, geometry_data.feature_id, ST_3DDistance(citydb.geometry_data.geometry, beam.reflection)
    FROM
        sensor_data.beam
    JOIN
        citydb.geometry_data
    ON ST_3DDWithin(citydb.geometry_data.geometry, beam.reflection, {distance_threshold})
    WHERE
        beam.patch_id = {current_patch_id};"
    );
    diesel::sql_query(query).execute(&mut connection).await?;

    //return Ok(());

    if beam_intersection {
        info!("Associating beam-model with patch_id: {current_patch_id}");
        let query = format!(
            "INSERT INTO sensor_data.association_beam_model (beam_id, feature_id, intersection)
SELECT DISTINCT b.id, g.feature_id, ST_3DIntersection(g.valid_geometry, b.reflection_line)
FROM
    (SELECT *
     FROM sensor_data.beam
     WHERE patch_id = {current_patch_id}) as b
JOIN
        (SELECT *
         FROM sensor_data.feature_geometry_data
         WHERE valid_geometry IS NOT NULL) as g
ON ST_3DIntersects(g.valid_geometry, b.reflection_line);"
        );
        diesel::sql_query(query).execute(&mut connection).await?;
    }

    Ok(())
}

async fn download_associate_points(
    mut connection: Object<AsyncPgConnection>,
    current_patch_id: i32,
    keep_temporary_table_entries: bool,
) -> Result<epoint::PointCloud, Error> {
    info!("Explode patch id: {current_patch_id}");

    /*let query = "SELECT PC_AsText(pa)
    FROM sensor_data.patches_associated
    LIMIT 1;";
        let a = diesel::sql_query(query).execute(&mut connection).await?;*/

    let query = format!("
INSERT INTO sensor_data.point_cloud_download (
    patch_id, x, y, z, point_id, timestamp_sec, timestamp_nanosec, intensity, beam_origin_x, beam_origin_y, beam_origin_z, beam_length, ros_message_id, ros_point_id, gml_id, gml_name, classname, surface_distance, intersection_angle)
SELECT
    b.patch_id,
    ST_X(b.reflection),
    ST_Y(b.reflection),
    ST_Z(b.reflection),
    b.point_id,
    b.timestamp_sec,
    b.timestamp_nanosec,
    b.intensity,
    ST_X(b.origin),
    ST_Y(b.origin),
    ST_Z(b.origin),
    b.length,
    b.ros_message_id,
    b.ros_point_id,
    cdb.objectid,
    cdb.name,
    cdb.classname,
    apm.distance,
    case when abm.intersection IS NULL then NULL else 1 end as intersection_angle
FROM sensor_data.beam as b
LEFT JOIN sensor_data.association_beam_model as abm ON b.id = abm.beam_id
LEFT JOIN sensor_data.association_point_model as apm ON b.id = apm.beam_id
LEFT JOIN
    (SELECT f.id as feature_id, f.objectid as objectid, p.val_string as name, oc.classname as classname
     FROM citydb.feature as f
     LEFT JOIN citydb.objectclass as oc ON f.objectclass_id = oc.id
     LEFT JOIN
        (SELECT *
        FROM citydb.property
        WHERE name = 'name') as p
     ON f.id = p.feature_id
     ) as cdb
ON apm.feature_id = cdb.feature_id
WHERE b.patch_id = {current_patch_id};");

    let _a = diesel::sql_query(query).execute(&mut connection).await?;

    info!("Download patch id: {current_patch_id}");
    let database_points: Vec<PointCloudDownloadEntry> =
        schema::sensor_data::point_cloud_download::dsl::point_cloud_download
            .filter(schema::sensor_data::point_cloud_download::patch_id.eq(current_patch_id))
            .load::<PointCloudDownloadEntry>(&mut connection)
            .await?;
    let number_of_points = database_points.len();
    info!("Number of points in patch {current_patch_id}: {number_of_points}");

    if !keep_temporary_table_entries {
        let num_deleted = diesel::delete(
            schema::sensor_data::point_cloud_download::dsl::point_cloud_download.filter(
                schema::sensor_data::point_cloud_download::dsl::patch_id.eq(current_patch_id),
            ),
        )
        .execute(&mut connection)
        .await?;
        info!("Deleted temporary entries of patch {current_patch_id}: point_cloud_download (number of rows: {num_deleted})");
    }

    let point_cloud = derive_point_cloud(database_points)?;
    Ok(point_cloud)
}

async fn explode_feature_geometry_data(
    mut connection: Object<AsyncPgConnection>,
) -> Result<(), Error> {
    let query = "INSERT INTO sensor_data.feature_geometry_data (geometry_data_id, feature_id, geometry, valid_geometry)
SELECT
    id,
    feature_id,
    geometry,
    case when ST_GeometryType(valid_geometry) = 'ST_Polygon' AND ST_IsPlanar(valid_geometry) then valid_geometry else null end as valid_geometry
FROM
    (SELECT
         id,
         feature_id,
         (ST_Dump(geometry_data.geometry)).geom::geometry(PolygonZ) as geometry,
         ST_MakeValid((ST_Dump(geometry_data.geometry)).geom::geometry(PolygonZ)) as valid_geometry,
         ST_AsText(ST_MakeValid((ST_Dump(geometry_data.geometry)).geom::geometry(PolygonZ)))
    FROM geometry_data
    WHERE
        ST_GeometryType(geometry_data.geometry) = 'ST_PolyhedralSurface' OR
        ST_GeometryType(geometry_data.geometry) = 'ST_MultiPolygon'
    ) as t;".to_string();

    let _a = diesel::sql_query(query).execute(&mut connection).await?;

    Ok(())
}

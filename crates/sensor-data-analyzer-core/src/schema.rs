// @generated automatically by Diesel CLI.

pub mod sensor_data {
    pub mod sql_types {
        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "geometry"))]
        pub struct Geometry;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "pcpatch"))]
        pub struct Pcpatch;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use crate::models::exports::*;
        use super::sql_types::Geometry;

        sensor_data.association_beam_model (id) {
            id -> Int8,
            beam_id -> Int8,
            feature_id -> Int8,
            intersection -> Geometry,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use crate::models::exports::*;

        sensor_data.association_point_model (id) {
            id -> Int8,
            beam_id -> Int8,
            feature_id -> Int8,
            distance -> Float8,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use crate::models::exports::*;
        use super::sql_types::Geometry;

        sensor_data.beam (id) {
            id -> Int8,
            patch_id -> Int4,
            point_id -> Int4,
            timestamp_sec -> Int4,
            timestamp_nanosec -> Int4,
            intensity -> Float8,
            origin -> Geometry,
            reflection -> Geometry,
            line -> Geometry,
            length -> Float8,
            reflection_line -> Geometry,
            ros_message_id -> Int4,
            ros_point_id -> Int4,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use crate::models::exports::*;
        use super::sql_types::Geometry;

        sensor_data.feature_geometry_data (id) {
            id -> Int8,
            geometry_data_id -> Int8,
            feature_id -> Int8,
            geometry -> Geometry,
            valid_geometry -> Nullable<Geometry>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use crate::models::exports::*;

        sensor_data.point_cloud_download (id) {
            id -> Int8,
            patch_id -> Int4,
            x -> Float8,
            y -> Float8,
            z -> Float8,
            point_id -> Int4,
            timestamp_sec -> Int4,
            timestamp_nanosec -> Int4,
            intensity -> Float8,
            beam_origin_x -> Float8,
            beam_origin_y -> Float8,
            beam_origin_z -> Float8,
            beam_length -> Float8,
            ros_message_id -> Int4,
            ros_point_id -> Int4,
            #[max_length = 256]
            gml_id -> Nullable<Varchar>,
            #[max_length = 1000]
            gml_name -> Nullable<Varchar>,
            #[max_length = 256]
            classname -> Nullable<Varchar>,
            surface_distance -> Nullable<Float8>,
            intersection_angle -> Nullable<Float8>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use crate::models::exports::*;
        use super::sql_types::Pcpatch;

        sensor_data.point_cloud_upload (id) {
            id -> Int4,
            pa -> Pcpatch,
        }
    }

    diesel::joinable!(association_beam_model -> beam (beam_id));
    diesel::joinable!(association_point_model -> beam (beam_id));

    diesel::allow_tables_to_appear_in_same_query!(
        association_beam_model,
        association_point_model,
        beam,
        feature_geometry_data,
        point_cloud_download,
        point_cloud_upload,
    );
}

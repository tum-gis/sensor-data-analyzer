pub mod sensor_data {
    use crate::schema::sensor_data::point_cloud_download;
    use crate::schema::sensor_data::point_cloud_upload;

    use diesel::{Identifiable, Insertable, Queryable};

    //     #[diesel(table_name = sensor_data::patches)]

    // pgpointcloud does not support a binary protocol: https://github.com/pgpointcloud/pointcloud/pull/246
    // https://github.com/diesel-rs/diesel/issues/2016

    #[derive(Debug, Clone, Insertable, Queryable, Identifiable)]
    #[diesel(table_name = point_cloud_upload)]
    pub struct Patches {
        pub id: i32,
        // pub pa: PcPath,
    }

    #[derive(Debug, Clone, Insertable, Queryable, Identifiable)]
    #[diesel(table_name = point_cloud_download)]
    pub struct PointCloudDownloadEntry {
        pub id: i64,
        pub patch_id: i32,
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub point_id: i32,
        pub timestamp_sec: i32,
        pub timestamp_nanosec: i32,
        pub intensity: f64,
        pub beam_origin_x: f64,
        pub beam_origin_y: f64,
        pub beam_origin_z: f64,
        pub beam_length: f64,
        pub ros_message_id: i32,
        pub ros_point_id: i32,
        pub gml_id: Option<String>,
        pub gml_name: Option<String>,
        pub classname: Option<String>,
        pub surface_distance: Option<f64>,
        pub intersection_angle: Option<f64>,
    }

    /*#[derive(Debug, Clone, FromSqlRow, AsExpression)]
    #[diesel(sql_type = crate::schema::sensor_data::sql_types::Pcpatch)]
    pub struct PcPath(pub String);
    impl ToSql<crate::schema::sensor_data::sql_types::Pcpatch, Pg> for PcPath {
        fn to_sql<'b>(&'b self, _out: &mut Output<'b, '_, Pg>) -> serialize::Result {
            //match *self {
            //    MyEnum::Foo => out.write_all(b"foo")?,
            //    MyEnum::Bar => out.write_all(b"bar")?,
            //}
            todo!("");
            Ok(IsNull::No)
        }
    }
    impl FromSql<crate::schema::sensor_data::sql_types::Pcpatch, Pg> for PcPath {
        fn from_sql(_bytes: PgValue<'_>) -> deserialize::Result<Self> {
            //match bytes.as_bytes() {
            //    b"foo" => Ok(MyEnum::Foo),
            //    b"bar" => Ok(MyEnum::Bar),
            //    _ => Err("Unrecognized enum variant".into()),
            //}
            todo!("");
        }
    }*/
}

pub mod exports {
    pub use super::sensor_data::*;
}

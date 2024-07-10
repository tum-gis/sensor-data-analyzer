DROP INDEX IF EXISTS sensor_data.temporary_points_geometry_index;
DROP INDEX IF EXISTS sensor_data.point_model_associations_point_index;

DROP TABLE IF EXISTS sensor_data.source_point_cloud;
DROP TABLE IF EXISTS sensor_data.source_point_cloud_exploded;
DROP TABLE IF EXISTS sensor_data.point_model_associations;
DROP TABLE IF EXISTS sensor_data.download_point_cloud_exploded;
--DROP TABLE IF EXISTS sensor_data.associations;

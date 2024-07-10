CREATE EXTENSION IF NOT EXISTS "postgis";
CREATE EXTENSION IF NOT EXISTS "postgis_sfcgal";
CREATE EXTENSION IF NOT EXISTS "pointcloud";
CREATE EXTENSION IF NOT EXISTS "pointcloud_postgis";

TRUNCATE pointcloud_formats;
INSERT INTO pointcloud_formats (pcid, srid, schema) VALUES (1, 25832,
'<?xml version="1.0" encoding="UTF-8"?>
<pc:PointCloudSchema xmlns:pc="http://pointcloud.org/schemas/PC/1.1"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <pc:dimension>
    <pc:position>1</pc:position>
    <pc:size>8</pc:size>
    <pc:description>X coordinate.</pc:description>
    <pc:name>x</pc:name>
    <pc:interpretation>double</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>2</pc:position>
    <pc:size>8</pc:size>
    <pc:description>Y coordinate.</pc:description>
    <pc:name>y</pc:name>
    <pc:interpretation>double</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>3</pc:position>
    <pc:size>8</pc:size>
    <pc:description>Z coordinate.</pc:description>
    <pc:name>z</pc:name>
    <pc:interpretation>double</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>4</pc:position>
    <pc:size>8</pc:size>
    <pc:description>The identifier.</pc:description>
    <pc:name>id</pc:name>
    <pc:interpretation>uint64_t</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>5</pc:position>
    <pc:size>8</pc:size>
    <pc:description>The timestamp in seconds.</pc:description>
    <pc:name>timestamp_sec</pc:name>
    <pc:interpretation>int64_t</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>6</pc:position>
    <pc:size>4</pc:size>
    <pc:description>The timestamp in nanoseconds since the last whole non-leap second.</pc:description>
    <pc:name>timestamp_nanosec</pc:name>
    <pc:interpretation>uint32_t</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>7</pc:position>
    <pc:size>4</pc:size>
    <pc:description>The intensity.</pc:description>
    <pc:name>intensity</pc:name>
    <pc:interpretation>float</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>8</pc:position>
    <pc:size>8</pc:size>
    <pc:description>Beam origin X coordinate of current laser shot.</pc:description>
    <pc:name>beam_origin_x</pc:name>
    <pc:interpretation>double</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>9</pc:position>
    <pc:size>8</pc:size>
    <pc:description>Beam origin Y coordinate of current laser shot.</pc:description>
    <pc:name>beam_origin_y</pc:name>
    <pc:interpretation>double</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>10</pc:position>
    <pc:size>8</pc:size>
    <pc:description>Beam origin Z coordinate of current laser shot.</pc:description>
    <pc:name>beam_origin_z</pc:name>
    <pc:interpretation>double</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>11</pc:position>
    <pc:size>4</pc:size>
    <pc:description>Message ID from the ROS bag.</pc:description>
    <pc:name>ros_message_id</pc:name>
    <pc:interpretation>uint32_t</pc:interpretation>
  </pc:dimension>
  <pc:dimension>
    <pc:position>12</pc:position>
    <pc:size>4</pc:size>
    <pc:description>Point ID from the ROS bag.</pc:description>
    <pc:name>ros_point_id</pc:name>
    <pc:interpretation>uint32_t</pc:interpretation>
  </pc:dimension>
  <pc:metadata>
    <Metadata name="compression">none</Metadata>
  </pc:metadata>
</pc:PointCloudSchema>');

--   <pc:dimension>
--     <pc:position>10</pc:position>
--     <pc:size>8</pc:size>
--     <pc:description>Associated city object.</pc:description>
--     <pc:name>city_object_id</pc:name>
--     <pc:interpretation>int64_t</pc:interpretation>
--   </pc:dimension>

-- see for custom data types: https://github.com/diesel-rs/diesel/blob/master/diesel_tests/tests/custom_types.rs#L9-L47
-- A table of points
--CREATE TABLE sensor_data.points (
--    id SERIAL PRIMARY KEY,
--    x DOUBLE PRECISION NOT NULL,
--    pt PCPOINT(1) NOT NULL
    --pt PCPOINT(1)
--);


CREATE TABLE sensor_data.feature_geometry_data (
    id BIGSERIAL PRIMARY KEY,
    geometry_data_id BIGINT NOT NULL,
    feature_id BIGINT NOT NULL,
    geometry geometry(PolygonZ) NOT NULL,
    valid_geometry geometry(PolygonZ)
);
CREATE INDEX idx_feature_geometry_data_geometry ON sensor_data.feature_geometry_data USING gist(geometry gist_geometry_ops_nd);
CREATE INDEX idx_feature_geometry_data_valid_geometry ON sensor_data.feature_geometry_data USING gist(valid_geometry gist_geometry_ops_nd);


CREATE TABLE sensor_data.point_cloud_upload (
    id SERIAL PRIMARY KEY,
    pa PCPATCH(1) NOT NULL
);


CREATE TABLE sensor_data.beam (
    id BIGSERIAL PRIMARY KEY,
    patch_id INT NOT NULL,
    point_id INT NOT NULL,
    timestamp_sec INT NOT NULL,
    timestamp_nanosec INT NOT NULL,
    intensity FLOAT NOT NULL,
    origin geometry(PointZ) NOT NULL,
    reflection geometry(PointZ) NOT NULL,
    line geometry(LinestringZ) NOT NULL,
    length FLOAT NOT NULL,
    reflection_line geometry(LinestringZ) NOT NULL,
    ros_message_id INT NOT NULL,
    ros_point_id INT NOT NULL
);
CREATE INDEX idx_beam_patch_id ON sensor_data.beam(patch_id);
CREATE INDEX idx_beam_point_id ON sensor_data.beam(point_id);
CREATE INDEX idx_beam_reflection ON sensor_data.beam USING gist(reflection gist_geometry_ops_nd);
CREATE INDEX idx_beam_reflection_line ON sensor_data.beam USING gist(reflection_line gist_geometry_ops_nd);


CREATE TABLE sensor_data.association_point_model (
    id BIGSERIAL PRIMARY KEY,
    beam_id BIGINT NOT NULL REFERENCES sensor_data.beam(id),
    feature_id BIGINT NOT NULL,
    distance FLOAT NOT NULL
);
CREATE INDEX idx_association_point_model_beam_id ON sensor_data.association_point_model(beam_id);
CREATE INDEX idx_association_point_model_feature_id ON sensor_data.association_point_model(feature_id);


CREATE TABLE sensor_data.association_beam_model (
    id BIGSERIAL PRIMARY KEY,
    beam_id BIGINT NOT NULL REFERENCES sensor_data.beam(id),
    feature_id BIGINT NOT NULL,
    intersection geometry(PointZ) NOT NULL
);
CREATE INDEX idx_association_beam_model_beam_id ON sensor_data.association_beam_model(beam_id);
CREATE INDEX idx_association_beam_model_feature_id ON sensor_data.association_beam_model(feature_id);

CREATE TABLE sensor_data.point_cloud_download (
    id BIGSERIAL PRIMARY KEY,
    patch_id INT NOT NULL,
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    z DOUBLE PRECISION NOT NULL,
    point_id INT NOT NULL,
    timestamp_sec INT NOT NULL,
    timestamp_nanosec INT NOT NULL,
    intensity FLOAT NOT NULL,
    beam_origin_x DOUBLE PRECISION NOT NULL,
    beam_origin_y DOUBLE PRECISION NOT NULL,
    beam_origin_z DOUBLE PRECISION NOT NULL,
    beam_length FLOAT NOT NULL,
    ros_message_id INT NOT NULL,
    ros_point_id INT NOT NULL,
    gml_id VARCHAR(256),
    gml_name VARCHAR(1000),
    classname VARCHAR(256),
    surface_distance FLOAT,
    intersection_angle FLOAT
);

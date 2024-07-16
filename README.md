# sensor-data-analyzer

The `sensor-data-analyzer` associates, analyzes, and enriches sensor data by leveraging semantic environment models.

The sensor data needs to be provided as a [ROS2](https://github.com/ros2) bag and the semantic environment model as [CityGML 3.0](https://www.ogc.org/standard/citygml/) dataset.
In order to associate, analyze, and enrich the sensor data in a scalable manner, the [3D City Database](https://github.com/3dcitydb/3dcitydb) is utilized and extended by a sensor data schema.

## Getting Started

Build and start the [extended-3dcitydb](other/extended-3dcitydb) provided as a Docker container.

Set up a [Rust development environment](https://www.rust-lang.org/tools/install) and install the [Diesel CLI](https://diesel.rs/guides/getting-started):

```bash
cargo install diesel_cli
```

Run the migration to set up the sensor data tables:

```bash
cd ./crates/sensor-data-analyzer-core
diesel migration run --database-url ${CITYDB_DATABASE_URL}
```

## Usage

To upload the ROS2 bag to the database, run:

```bash
cargo run -r -- upload-rosbag \
    --rosbag-directory-path /path/to/rosbag \
    --ecoord-file-path /path/to/additional/ecoord \
    --maximum-number-connections 10 \
    --start-time-offset 20s --total-duration 4s
```

To associate the individual sensor observations with objects from the semantic model, run:

```bash
cargo run -r -- associate --distance-threshold 0.2
```

In order to download the associated sensor data, run:

```bash
cargo run -r -- download --directory-path /path/downloaded/point/clouds
```

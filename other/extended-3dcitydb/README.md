# extended-3dcitydb

Docker container of the [3D City Database](https://github.com/3dcitydb/3dcitydb) extended with [pgPointCLoud](https://pgpointcloud.github.io/pointcloud/concepts/index.html) and a dedicated sensor data schema.

## Getting started

Build the container:

```bash
docker build -t extended-3dcitydb .
```

Run the container:

```bash
docker run --name extended-3dcitydb -p 5432:5432 -d \
    -e SRID=25832 \
    -e HEIGHT_EPSG=7837 \
    -e SRS_NAME=urn:ogc:def:crs:EPSG::25832 \
    -e POSTGRES_DB=citydb \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGIS_SFCGAL=true \
  extended-3dcitydb
```

## Admin

If you need an admin interface, start [pgAdmin](https://www.pgadmin.org/):

```bash
docker run --name pgadmin -p 8080:80 -d \
  -e PGADMIN_DEFAULT_EMAIL=postgres@example.com \
  -e PGADMIN_DEFAULT_PASSWORD=postgres \
  dpage/pgadmin4
```

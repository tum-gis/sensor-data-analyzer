
# Fetch & build stage #########################################################
# ARGS
ARG BASEIMAGE_TAG='16-3.4'
ARG BUILDER_IMAGE_TAG='17-jdk-jammy'

# Base image
FROM eclipse-temurin:${BUILDER_IMAGE_TAG} AS builder

# Copy source code
WORKDIR /build
COPY ./3dcitydb /build

# Build
RUN chmod u+x ./gradlew && ./gradlew installDist


# Runtime stage ###############################################################
FROM postgres:16  AS runtime
# FROM postgis/postgis:16-3.2

# install: https://github.com/pgpointcloud/pointcloud/blob/master/docker/Dockerfile

ENV POSTGRES_VERSION 16
ENV POSTGIS_VERSION 3

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    postgis \
    postgresql-${POSTGRES_VERSION}-postgis-${POSTGIS_VERSION} \
    postgresql-${POSTGRES_VERSION}-postgis-${POSTGIS_VERSION}-scripts

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    git \
    ca-certificates \
    build-essential \
    autoconf \
    automake \
    cmake \
    zlib1g-dev \
    postgresql-server-dev-all \
    libxml2-dev \
    && rm -rf /var/lib/apt/lists/* \
    && git clone https://github.com/verma/laz-perf.git \
    && cd laz-perf \
    && cmake . \
    && make \
    && make install \
    && cd .. \
    && rm -r laz-perf \
    && git clone https://github.com/pgpointcloud/pointcloud \
    && cd pointcloud \
    && ./autogen.sh \
    && ./configure --with-lazperf=/usr/local --with-pgconfig=/usr/lib/postgresql/${POSTGRES_VERSION}/bin/pg_config CFLAGS="-Wall -Werror -O2 -g" \
    && make \
    && make install \
    && apt-get purge -y --auto-remove \
    git \
    ca-certificates \
    build-essential \
    autoconf \
    automake \
    cmake \
    zlib1g-dev \
    postgresql-server-dev-all \
    libxml2-dev

RUN mkdir -p /docker-entrypoint-initdb.d
COPY ./pgpointcloud/docker/initdb-pgpointcloud.sh /docker-entrypoint-initdb.d/05_pgpointcloud.sh

# see: https://github.com/3dcitydb/3dcitydb/blob/master/postgresql/Dockerfile
ENV POSTGIS_SFCGAL true

# Copy SQL scripts
WORKDIR /3dcitydb
COPY --from=builder /build/build/install/3dcitydb/postgresql/sql-scripts .
COPY --from=builder /build/build/install/3dcitydb/postgresql/docker-scripts/3dcitydb-initdb.sh /docker-entrypoint-initdb.d/


COPY ./sensor-data-schema.sh /docker-entrypoint-initdb.d/15_sensordata-initschema.sh

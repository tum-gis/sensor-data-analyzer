#!/usr/bin/env bash

set -e;
# psql should stop on error
psql=( psql -v ON_ERROR_STOP=1 )

echo "Create sensor_data schema in database '$POSTGRES_DB' ..."
"${psql[@]}" -d "$POSTGRES_DB" -c "CREATE SCHEMA IF NOT EXISTS sensor_data;"

#!/usr/bin/env bash
set -x
set -eo pipefail

SKIP_DOCKER=0

function parse_args() {
    if [ $1 = "--skip-docker" ]; then
        SKIP_DOCKER=1
    fi
}

parse_args $@

# Make sure dependencies are installed
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "psql must be installed"
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "sqlx is not installed"
    exit 1
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD=${POSTGRES_PASSWORD:=postgres}
DB_NAME=${POSTGRES_DB:=newsletter}
DB_PORT=${POSTGRES_PORT:=5432}

# Start container
if [ $SKIP_DOCKER -eq 0 ]; then
    docker run \
        --name newsletter-db \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d postgres \
        postgres -N 1000
fi

# Wait for container to be ready
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "postgres is not ready yet"
    sleep 1
done

>&2 echo "postgres is ready on port ${DB_PORT}"

# Prep sqlx
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

>&2 echo "Postgres is ready"

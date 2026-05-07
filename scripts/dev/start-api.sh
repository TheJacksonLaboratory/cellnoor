#!/usr/bin/env sh

set -euo pipefail

docker_compose="docker compose --file compose.yaml --file compose.dev.yaml"

function cleanup_docker() {
    $docker_compose rm --force --stop --volumes
    $docker_compose volumes --format json | jq '.[].Name' --slurp | xargs docker volume rm
}
trap cleanup_docker EXIT

$docker_compose up db migrate --detach

# The build script cellnoor-schema/build.rs calls the diesel-cli, which may need a connection to a database. We
# provide the URL of the database spun up in restart-compilation-db.sh via an environment variable, which diesel picks
# up automatically
export CELLNOOR_DB_URL="postgres://app:p@localhost:5432/postgres"
export CELLNOOR_AUTH_SECRET=""
export CELLNOOR_AUTH_URL=""
export CELLNOOR_ADDRESS="localhost:8000"
export CELLNOOR_APP_URL="localhost:8000"

cargo run --manifest-path crates/Cargo.toml --package cellnoor --features ssr $@

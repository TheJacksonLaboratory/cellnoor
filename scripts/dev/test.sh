#!/usr/bin/env bash

set -euo pipefail

docker_compose="docker compose --file compose.yaml --file compose.dev.yaml"

function cleanup_docker() {
    $docker_compose rm --force --stop --volumes
    $docker_compose volumes --format json | jq '.[].Name' --slurp | xargs --no-run-if-empty docker volume rm
}
trap cleanup_docker EXIT

$docker_compose up db migrate --detach

# docker compose reads .env on its own, but this shell needs the same values
if [ -f .env ]; then
    set -o allexport
    source .env
    set +o allexport
fi

# The tests must connect as 'app' like the application does: row-level security doesn't apply to the superuser
CELLNOOR_TEST_DB_URL="postgres://app:${CELLNOOR_APP_DB_PASSWORD}@localhost:5432/postgres" cargo test --workspace --manifest-path crates/Cargo.toml --all-features "$@"

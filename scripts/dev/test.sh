#!/usr/bin/env bash

set -euo pipefail

docker_compose="docker compose --file compose.yaml --file compose.dev.yaml"

function cleanup_docker() {
    $docker_compose rm --force --stop --volumes
    $docker_compose volumes --format json | jq '.[].Name' --slurp | xargs --no-run-if-empty docker volume rm
}
trap cleanup_docker EXIT

$docker_compose up db migrate --detach

CELLNOOR_TEST_DB_URL="postgres://postgres:p@localhost:5432/postgres" cargo test --workspace --manifest-path crates/Cargo.toml --all-features "$@"

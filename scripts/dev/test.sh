#!/usr/bin/env sh

set -euo pipefail

docker_compose="docker compose --env-file .env.dev --file compose.yaml --file compose.dev.yaml"

function cleanup_docker() {
    $docker_compose stop >/dev/null
    $docker_compose rm --volumes >/dev/null
}
trap cleanup_docker EXIT

$docker_compose up db migrate --detach

CELLNOOR_TEST_DB_URL="postgres://postgres:p@localhost:5432/postgres" cargo test --workspace --manifest-path crates/Cargo.toml --all-features $@

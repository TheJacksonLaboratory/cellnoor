#!/usr/bin/env sh

set -euo pipefail

function cleanup_docker() {
    docker stop cellnoor-api_test >/dev/null
    docker rm cellnoor-api_test --volumes >/dev/null
}
trap cleanup_docker EXIT

docker_compose="docker compose --env-file .env.dev --file compose.yaml --file compose.dev.yaml"

$docker_compose up db migrate --detach

cd crates && CELLNOOR_TEST_DB_URL="postgres://postgres:p@localhost:5432/postgres" cargo test --workspace --all-features $@

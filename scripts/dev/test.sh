#!/usr/bin/env sh

set -euo pipefail

docker_compose="docker compose --env-file .env.dev --file compose.yaml --file compose.dev.yaml"

$docker_compose up db migrate --detach

cd crates && CELLNOOR_TEST_DB_URL="postgres://postgres:p@localhost:5432/postgres" cargo test --workspace --all-features $@

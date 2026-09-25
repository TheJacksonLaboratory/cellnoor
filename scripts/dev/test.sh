#!/usr/bin/env bash

set -euo pipefail

trap 'scripts/dev/cleanup-docker.sh --yes' EXIT

export CELLNOOR_DB_ROOT_PASSWORD="p"
export CELLNOOR_API__DB__PASSWORD="$CELLNOOR_DB_ROOT_PASSWORD"
export CELLNOOR_AUTH__DB_PASSWORD="$CELLNOOR_DB_ROOT_PASSWORD"

scripts/dev/compose.sh up db migrate --detach

CELLNOOR_TEST_DB_URL="postgres://app:${CELLNOOR_API__DB__PASSWORD}@localhost:5432/postgres" cargo test --workspace --manifest-path crates/Cargo.toml --all-features "$@"

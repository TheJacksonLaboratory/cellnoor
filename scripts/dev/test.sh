#!/usr/bin/env bash

set -euo pipefail

trap 'scripts/dev/cleanup-docker.sh --yes' EXIT

scripts/dev/compose.sh up db migrate --detach

# docker compose reads .env on its own, but this shell needs the same values
if [ -f .env ]; then
    set -o allexport
    source .env
    set +o allexport
fi

# The tests must connect as 'app' like the application does: row-level security doesn't apply to the superuser
CELLNOOR_TEST_DB_URL="postgres://app:${CELLNOOR_APP_DB_PASSWORD}@localhost:5432/postgres" cargo test --workspace --manifest-path crates/Cargo.toml --all-features "$@"

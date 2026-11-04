#!/usr/bin/env bash

set -euo pipefail

trap 'scripts/dev/cleanup-docker.sh --yes' EXIT

export CELLNOOR_DB_ROOT_PASSWORD="p"
export CELLNOOR_API__DB__PASSWORD="$CELLNOOR_DB_ROOT_PASSWORD"
export CELLNOOR_AUTH__DB__PASSWORD="$CELLNOOR_DB_ROOT_PASSWORD"

scripts/dev/compose.sh up db migrate --detach

CELLNOOR_TEST_DB_URL="postgres://app:${CELLNOOR_API__DB__PASSWORD}@localhost:5432/postgres" cargo test --workspace --manifest-path crates/Cargo.toml --all-features "$@"

CELLNOOR_AUTH__PUBLIC_BASE_URL="localhost" CELLNOOR_AUTH__DB__HOST="localhost" CELLNOOR_AUTH__DB__NAME="postgres" CELLNOOR_AUTH__AUTH_SECRET="cellnoor-auth-test-secret-0123456789abcdef" CELLNOOR_AUTH__MICROSOFT_ENTRA_CLIENT_ID="cellnoor-auth-test-client" CELLNOOR_AUTH__MICROSOFT_ENTRA_CLIENT_SECRET="cellnoor-auth-test-client-secret" bun --cwd=packages/cellnoor-auth test

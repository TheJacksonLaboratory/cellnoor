#! /usr/bin/env bash

# Because `docker compose` can be slow, this script starts all the services of this application in a dev build
set -euo pipefail

sockets=$(mktemp -d)

cleanup() {
	scripts/dev/cleanup-docker.sh --yes
	rm -rf "$sockets"
	kill 0
}
trap cleanup EXIT

set -o allexport
source .env
set +o allexport

# Common configuration parameters
host="cellnoor.localhost"
auth_secret=$(openssl rand -base64 32)
db_host="localhost"
db_name="postgres"
db_password="p"
static_files_dir="$PWD/.static-files"

# db configuration
export CELLNOOR_DB_ROOT_PASSWORD="$db_password"

# cellnoor-auth configuration
export CELLNOOR_AUTH__DB_HOST="$db_host"
export CELLNOOR_AUTH__DB_PASSWORD="$db_password"
export CELLNOOR_AUTH__DB_NAME="$db_name"
export CELLNOOR_AUTH__AUTH_SECRET="$auth_secret"
export CELLNOOR_AUTH__UNIX_DOMAIN_SOCKET="$sockets/auth.sock"
export CELLNOOR_AUTH__PUBLIC_BASE_URL="https://$host"

# cellnoor-api configuration
export CELLNOOR_API__DB__HOST="$db_host"
export CELLNOOR_API__DB__PASSWORD="$db_password"
export CELLNOOR_API__DB__DBNAME="$db_name"
export CELLNOOR_API__AUTH_SECRET="$auth_secret"
export CELLNOOR_API__STATIC_FILES_DIR="$static_files_dir"
export CELLNOOR_API__LISTEN_ON="$sockets/api.sock"

# Caddy configuration
export CELLNOOR_CADDY__PUBLIC_BASE_URL="$host"
export CELLNOOR_CADDY__API_SOCKET="$CELLNOOR_API__LISTEN_ON"
export CELLNOOR_CADDY__AUTH_SOCKET="$CELLNOOR_AUTH__UNIX_DOMAIN_SOCKET"
export CELLNOOR_CADDY__STATIC_FILES_DIR="$static_files_dir"
export CELLNOOR_CADDY__UI_DIR="$PWD/packages/cellnoor-ui/build"

mkdir -p "$static_files_dir"

scripts/dev/compose.sh up db --wait
scripts/dev/compose.sh up migrate

cargo run --manifest-path crates/Cargo.toml --package cellnoor-api --bin cellnoor-api &
(cd packages/cellnoor-auth && bun install && bun index.ts) &
bun run --cwd packages/cellnoor-ui build --watch &
caddy run --config caddy/Caddyfile --watch &
echo "app running on https://$host"

wait

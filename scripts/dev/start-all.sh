#! /usr/bin/env bash

# Because `docker compose` can be slow, this script starts all the services of this application in a dev build
set -euo pipefail

sockets=$(mktemp -d)
logs=$(mktemp -d)

quiet() {
	local name=$1
	shift
	"$@" >"$logs/$name.log" 2>&1 || {
		cat "$logs/$name.log" >&2
		return 1
	}
}

cleanup() {
	trap '' TERM
	kill 0
	wait
	tail -n +1 "$logs"/{api,auth,ui-watch,caddy}.log 2>/dev/null || true
	quiet cleanup scripts/dev/cleanup-docker.sh --yes
	rm -rf "$sockets" "$logs"
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

quiet colima colima start

# Set up the db
quiet db scripts/dev/compose.sh up db --wait
quiet migrate scripts/dev/compose.sh run --rm migrate

# Build the 3 services in the foreground so that we fail early if something is wrong
quiet api-build cargo build --manifest-path crates/Cargo.toml --package cellnoor-api --bin cellnoor-api
quiet ui-build bun run --bun --cwd packages/cellnoor-ui check
quiet auth-install bun install --cwd packages/cellnoor-auth

./crates/target/debug/cellnoor-api >"$logs/api.log" 2>&1 &
bun --cwd packages/cellnoor-auth index.ts >"$logs/auth.log" 2>&1 &
bun run --cwd packages/cellnoor-ui build --watch >"$logs/ui-watch.log" 2>&1 &
caddy run --config caddy/Caddyfile --watch >"$logs/caddy.log" 2>&1 &

until [[ -S $CELLNOOR_API__LISTEN_ON && -S $CELLNOOR_AUTH__UNIX_DOMAIN_SOCKET ]]; do
	[[ $(jobs -rp | wc -l) -eq 4 ]] || exit 1
	sleep 0.1
done

echo "cellnoor running on https://$host"

while [[ $(jobs -rp | wc -l) -eq 4 ]]; do sleep 1; done

exit 1

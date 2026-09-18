#! /usr/bin/env bash

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

# The production Caddyfile is reused as-is: every upstream in it is a unix socket, and both the api and the auth
# service can bind one, so dev only has to fill in the placeholders. An explicit port 80 turns off automatic HTTPS.
host=cellnoor.localhost
export CELLNOOR_PUBLIC_BASE_URL="$host:80"
export CELLNOOR_API_SOCKET="$sockets/api.sock"
export CELLNOOR_AUTH_SOCKET="$sockets/auth.sock"
export CELLNOOR_STATIC_FILES_DIR="$CELLNOOR_API__STATIC_FILES_DIR"
export CELLNOOR_UI_DIR="$PWD/packages/cellnoor-ui/build"

export CELLNOOR_API__LISTEN_ON="$CELLNOOR_API_SOCKET"
export CELLNOOR_AUTH__UNIX_DOMAIN_SOCKET="$CELLNOOR_AUTH_SOCKET"
export CELLNOOR_AUTH__PUBLIC_AUTH_URL="http://$host/api/auth"

scripts/dev/compose.sh up db migrate --wait

cargo run --manifest-path crates/Cargo.toml --package cellnoor-api --bin cellnoor-api
& (cd packages/cellnoor-auth && bun install && bun index.ts)
& bun run --cwd packages/cellnoor-ui build --watch
& caddy run --config caddy/Caddyfile
& echo "app on http://$host"
wait

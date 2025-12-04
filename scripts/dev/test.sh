#!/usr/bin/env sh

set -euo pipefail

function cleanup_docker() {
    docker stop scamplers-api_test >/dev/null
    docker rm scamplers-api_test --volumes >/dev/null
}
trap cleanup_docker EXIT

# Note that this database has port 5432 mapped to the host machine's port 5433, since we know the compilation database
# (started in restart-compilation-db.sh) is using port 5432
docker run --name scamplers-api_test --env POSTGRES_PASSWORD=p --publish 5433:5432 --detach postgres:18-alpine

# Thanks ChatGPT
until docker exec --user postgres scamplers-api_test pg_isready >/dev/null 2>&1; do
    sleep 0.1
done

export SCAMPLERS_CONFIG_DIR=".."
export SCAMPLERS_DB_ROOT_USER=postgres
export SCAMPLERS_DB_ROOT_PASSWORD="p"
export SCAMPLERS_API_DB_PASSWORD="p"
export SCAMPLERS_UI_DB_PASSWORD=""
export SCAMPLERS_DB_HOST=localhost
export SCAMPLERS_DB_PORT=5433
export SCAMPLERS_DB_NAME=postgres
export SCAMPLERS_API_KEY_PREFIX_LENGTH=8
export SCAMPLERS_API_HOST=localhost
export SCAMPLERS_API_PORT=8000

cargo test --workspace $@

#!/usr/bin/env sh

set -euo pipefail

function cleanup_docker() {
    docker stop cellnoor-api_test >/dev/null
    docker rm cellnoor-api_test --volumes >/dev/null
}
trap cleanup_docker EXIT

# Note that this database has port 5432 mapped to the host machine's port 5433, since we know the compilation database
# (started in restart-compilation-db.sh) is using port 5432
docker run --name cellnoor-api_test --env POSTGRES_PASSWORD=p --publish 5433:5432 --detach postgres:18-alpine

# Thanks ChatGPT
until docker exec --user postgres cellnoor-api_test pg_isready >/dev/null 2>&1; do
    sleep 0.1
done

export CELLNOOR_CONFIG_DIR=".."
export CELLNOOR_DB_ROOT_USER=postgres
export CELLNOOR_DB_ROOT_PASSWORD="p"
export CELLNOOR_API_DB_PASSWORD="p"
export CELLNOOR_UI_DB_PASSWORD=""
export CELLNOOR_DB_HOST=localhost
export CELLNOOR_DB_PORT=5433
export CELLNOOR_DB_NAME=postgres
export CELLNOOR_API_KEY_PREFIX_LENGTH=8
export CELLNOOR_API_HOST=localhost
export CELLNOOR_API_PORT=8000

cargo test --workspace $@

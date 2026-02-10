#!/usr/bin/env sh

set -euo pipefail

function cleanup_docker() {
    docker stop cellnoor-api_test >/dev/null
    docker rm cellnoor-api_test --volumes >/dev/null
}
trap cleanup_docker EXIT

# Note that this database has port 5432 mapped to the host machine's port 5433, since we know the compilation database
# (started in restart-compilation-db.sh) is using port 5432
docker run --name cellnoor-api_test --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5433:5432 --detach postgres:18-alpine

until diesel database setup --config-file crates/cellnoor-schema/diesel.toml --database-url postgres://postgres@localhost:5433/postgres --migration-dir crates/cellnoor-schema/migrations >/dev/null 2>&1; do
    sleep 0.1
done

export CELLNOOR_DB_ROOT_USER=postgres
export CELLNOOR_DB_ROOT_PASSWORD=""
export CELLNOOR_API_DB_PASSWORD=""

cargo test --workspace --all-features $@

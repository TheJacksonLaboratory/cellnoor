#!/usr/bin/env sh

set -euo pipefail

function cleanup_docker() {
    docker stop cellnoor-dev-db >/dev/null
    docker rm cellnoor-dev-db --volumes >/dev/null
}
trap cleanup_docker EXIT

# Note that this database has port 5432 mapped to the host machine's port 5433, since we know the compilation database
# (started in restart-compilation-db.sh) is using port 5432
docker run --name cellnoor-dev-db --env POSTGRES_PASSWORD=p --publish 5433:5432 --detach postgres:18-alpine

# Thanks ChatGPT
until docker exec --user postgres cellnoor-dev-db pg_isready >/dev/null 2>&1; do
    sleep 0.1
done

# The build script cellnoor-schema/build.rs calls the diesel-cli, which may need a connection to a database. We
# provide the URL of the database spun up in restart-compilation-db.sh via an environment variable, which diesel picks
# up automatically
DATABASE_URL="postgres://postgres@localhost:5432/cellnoor-compilation" CELLNOOR_DB_ROOT_USER="postgres" CELLNOOR_DB_ROOT_PASSWORD="p" CELLNOOR_API_DB_PASSWORD="p" CELLNOOR_PUBLIC_API_URL="http://localhost:8000/api" CELLNOOR_PUBLIC_UI_URL="http://localhost:5173" cargo run --bin cellnoor-api $@

#!/usr/bin/env sh

set -euo pipefail

function cleanup_docker() {
    docker stop scamplers-dev >/dev/null
    docker rm scamplers-dev --volumes >/dev/null
}
trap cleanup_docker EXIT

# Note that this database has port 5432 mapped to the host machine's port 5433, since we know the compilation database
# (started in restart-compilation-db.sh) is using port 5432
docker run --name scamplers-dev --env POSTGRES_PASSWORD=p --publish 5433:5432 --detach postgres:18-alpine

# Thanks ChatGPT
until docker exec --user postgres scamplers-dev pg_isready >/dev/null 2>&1; do
    sleep 0.1
done

# The build script scamplers-schema/build.rs calls the diesel-cli, which may need a connection to a database. We
# provide the URL of the database spun up in restart-compilation-db.sh via an environment variable, which diesel picks
# up automatically
DATABASE_URL="postgres://postgres@localhost:5432/scamplers-compilation" cargo build --package scamplers-api

# Run scamplers-api, passing along any command-line arguments to override configuration values
./target/debug/scamplers-api $@

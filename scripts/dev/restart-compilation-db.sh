#!/usr/bin/env sh

set -euo pipefail

docker rm cellnoor-compilation --force --volumes
docker run --name cellnoor-compilation --env POSTGRES_HOST_AUTH_METHOD=trust --env POSTGRES_DB=cellnoor-compilation --publish 5432:5432 --detach postgres:18-alpine

#!/usr/bin/env bash

set -euo pipefail

source .env

PUBLIC_AUTH_URL=$CELLNOOR__PUBLIC_AUTH_URL AUTH_SECRET=$CELLNOOR__AUTH_SECRET API_URL=http://localhost:8000 bun run --bun --cwd=packages/cellnoor-ui --install=force --sql-preconnect --env-file=../.env dev

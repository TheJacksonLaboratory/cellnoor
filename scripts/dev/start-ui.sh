#!/usr/bin/env sh

set -euo pipefail

CELLNOOR_PUBLIC_URL=http://localhost:5173 bun run --bun --cwd=cellnoor-ui --install=force --sql-preconnect --env-file=../.env dev

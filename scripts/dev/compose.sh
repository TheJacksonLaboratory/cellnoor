#! /usr/bin/env bash

# The one place that knows how to invoke compose for development.

set -euo pipefail

args=(--file compose.yaml --file compose.dev.yaml)

# CI passes the secrets as environment variables instead, and has no .env.compose
if [[ -f .env.compose ]]; then
    args=(--env-file .env.compose "${args[@]}")
fi

docker compose "${args[@]}" "$@"

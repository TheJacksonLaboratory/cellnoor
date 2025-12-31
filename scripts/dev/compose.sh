#! /usr/bin/env bash

set -euo pipefail

docker compose --env-file .env.compose --file compose.yaml --file compose.dev.yaml $@

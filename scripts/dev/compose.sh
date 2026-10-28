#! /usr/bin/env bash

set -euo pipefail

docker compose --file compose.yaml --file compose.dev.yaml "$@"

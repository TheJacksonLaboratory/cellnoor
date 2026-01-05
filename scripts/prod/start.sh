#! /usr/bin/env bash

set -euo pipefail

docker compose --env-file .env.prod --file compose.yaml --file compose.prod.yaml up --build

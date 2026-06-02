#! /usr/bin/env bash

set -euo pipefail

docker_compose="docker compose --env-file .env --file compose.yaml --file compose.dev.yaml"

$docker_compose up --build

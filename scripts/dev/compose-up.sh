#! /usr/bin/env bash

set -euo pipefail

docker_compose="docker compose --env-file .env.compose --file compose.yaml --file compose.dev.yaml"

$docker_compose up --build

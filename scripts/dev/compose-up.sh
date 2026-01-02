#! /usr/bin/env bash

set -euo pipefail

docker_compose="docker compose --env-file .env.compose --file compose.yaml --file compose.dev.yaml"

function cleanup_docker() {
    $docker_compose rm --stop --force --volumes
    $docker_compose volumes --format json | jq '.[].Name' --slurp | xargs docker volume rm
}

trap cleanup_docker EXIT

$docker_compose up --build

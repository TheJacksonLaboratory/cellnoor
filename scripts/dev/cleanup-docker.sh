#! /usr/bin/env bash

set -euo pipefail

first_arg="${1:-}"
need_confirmation=true
if [[ $first_arg == "-y" || $first_arg == "--yes" ]]; then
	need_confirmation=false
fi

if [[ $need_confirmation == true ]]; then
	prompt="This script will remove all containers and their associated volumes, meaning the database will be deleted. Continue? [y/N] "
	read -r -p "$prompt" reply
	if [[ ! $reply =~ ^[Yy]$ ]]; then
		echo "Aborted."
		exit 0
	fi
fi

docker_compose="docker compose --env-file .env.compose --file compose.yaml --file compose.dev.yaml"

$docker_compose rm --stop --force --volumes
$docker_compose volumes --format json | jq '.[].Name' --slurp | xargs docker volume rm

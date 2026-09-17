#! /usr/bin/env bash

set -euo pipefail

compose=scripts/dev/compose.sh

first_arg="${1:-}"
if [[ $first_arg != "-y" && $first_arg != "--yes" ]]; then
    prompt="This script will remove all containers and their associated volumes, meaning the database will be deleted. Continue? [y/N] "
    read -r -p "$prompt" reply
    if [[ ! $reply =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

$compose rm --stop --force --volumes
$compose volumes --format json | jq '.[].Name' --slurp | xargs --no-run-if-empty docker volume rm

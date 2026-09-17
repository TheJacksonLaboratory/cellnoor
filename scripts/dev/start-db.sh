#! /usr/bin/env bash

set -euo pipefail

trap 'scripts/dev/cleanup-docker.sh --yes' EXIT

scripts/dev/compose.sh up db migrate

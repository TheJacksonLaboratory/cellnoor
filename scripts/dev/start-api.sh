#!/usr/bin/env bash

set -euo pipefail

trap 'scripts/dev/cleanup-docker.sh --yes' EXIT

scripts/dev/compose.sh up db migrate --detach

cargo run --manifest-path crates/Cargo.toml --package cellnoor --bin cellnoor "$@"

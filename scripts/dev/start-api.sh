#!/usr/bin/env bash

set -euo pipefail

cargo run --manifest-path crates/Cargo.toml --package cellnoor-api --bin cellnoor-api "$@"

#!/usr/bin/env sh

set -euo pipefail

cargo run --manifest-path crates/Cargo.toml --package cellnoor --features ssr --bin openapi >openapi.json

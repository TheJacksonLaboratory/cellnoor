#!/usr/bin/env bash

set -euo pipefail

DATABASE_URL="postgres://postgres@localhost:5432/cellnoor-compilation" cargo clippy --fix --allow-dirty --workspace
cargo +nightly fmt

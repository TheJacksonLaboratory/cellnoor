#!/usr/bin/env bash

set -euo pipefail

DATABASE_URL="postgres://postgres@localhost:5432/scamplers-compilation" cargo clippy --fix --allow-dirty --workspace
cargo +nightly fmt

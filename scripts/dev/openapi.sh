#!/usr/bin/env bash

set -euo pipefail

cargo run --manifest-path crates/Cargo.toml --package cellnoor-api --bin openapi >openapi.json

bunx openapi-typescript openapi.json -o packages/cellnoor-client/cellnoor-types.ts --root-types --root-types-no-schema-prefix --enum-values --default-non-nullable false

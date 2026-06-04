#!/usr/bin/env sh

set -euo pipefail

cargo run --manifest-path crates/Cargo.toml --package cellnoor --features ssr --bin openapi >openapi.json

bunx openapi-typescript openapi.json -o packages/cellnoor-ui/src/lib/cellnoor-types.d.ts --root-types --root-types-no-schema-prefix --enum-values --default-non-nullable false

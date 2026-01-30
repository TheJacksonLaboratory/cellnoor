#!/usr/bin/env bash

set -euo pipefail

cargo run --bin openapi > openapi.json
bunx openapi-typescript openapi.json -o pkgs/cellnoor-client/api.d.ts --root-types --root-types-no-schema-prefix
bun run --bun --cwd=cellnoor-ui check
bun run --bun --cwd=cellnoor-ui fmt

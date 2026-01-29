#!/usr/bin/env bash

set -euo pipefail

cargo run --bin openapi > openapi.json
bunx openapi-typescript openapi.json pkgs/cellnoor-client
bun run --bun --cwd=cellnoor-ui check
bun run --bun --cwd=cellnoor-ui fmt

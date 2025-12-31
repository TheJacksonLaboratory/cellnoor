#!/usr/bin/env bash

set -euo pipefail

mv pkgs/cellnoor-types/package.json cellnoor-types.package.json
rm -rf cellnoor-types/*
cargo run --package cellnoor-typescript
mv cellnoor-types.package.json pkgs/cellnoor-types/package.json
bun run --bun --cwd=cellnoor-ui check
bun run --bun --cwd=cellnoor-ui fmt

#!/usr/bin/env bash

set -euo pipefail

mv scamplers-types/package.json scamplers-types.package.json
rm -rf scamplers-types/*
cargo run --package scamplers-typescript
mv scamplers-types.package.json scamplers-types/package.json
bun run --bun --cwd=scamplers-ui check
bun run --bun --cwd=scamplers-ui fmt

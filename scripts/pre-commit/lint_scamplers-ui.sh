#!/usr/bin/env bash

set -euo pipefail

cargo run --package scamplers-typescript
bun run --bun --cwd=scamplers-ui check
deno fmt scamplers-ui

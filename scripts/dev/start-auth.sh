#!/usr/bin/env sh

set -euo pipefail

cd packages/cellnoor-auth
bun install && bun --env-file=../../.env src/index.ts

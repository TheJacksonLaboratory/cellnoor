#!/usr/bin/env bash

set -euo pipefail

cd packages/cellnoor-auth
bun install && bun --env-file=../../.env src/index.ts

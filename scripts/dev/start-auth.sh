#!/usr/bin/env sh

set -euo pipefail

bun --cwd=packages/cellnoor-auth --install=force --env-file=../../.env.rewrite src/index.ts

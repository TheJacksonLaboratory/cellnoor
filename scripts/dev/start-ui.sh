#!/usr/bin/env bash

set -euo pipefail

source .env

bun run dev --cwd=packages/cellnoor-ui

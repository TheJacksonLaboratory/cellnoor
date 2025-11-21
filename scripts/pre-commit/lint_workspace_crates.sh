#!/usr/bin/env bash

set -euo pipefail

cargo clippy --fix --allow-dirty --workspace --exclude scamplers-api --exclude scamplers-typescript

#! /usr/bin/env bash

set -euo pipefail

. venv/bin/activate
git add .
git commit --message "$@"
git push

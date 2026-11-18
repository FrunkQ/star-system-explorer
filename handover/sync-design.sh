#!/usr/bin/env bash
# Refresh handover/design from the project's shared design folder.
set -euo pipefail
src="${1:-/mnt/project-files/sse4}"
cd "$(dirname "$0")"
cp "$src"/*.md design/
git status --short design/

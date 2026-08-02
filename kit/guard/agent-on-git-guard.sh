#!/usr/bin/env bash
# Compatibility path (older hooks used this filename). Delegates to agent-on-git-guard.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
exec bash "${HERE}/agent-on-git-guard"

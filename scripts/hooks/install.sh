#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

current="$(git config --get core.hooksPath || true)"
if [[ -n "$current" && "$current" != .githooks ]]; then
    printf 'error: core.hooksPath is already %s; integrate the pre-commit hook there before changing it.\n' "$current" >&2
    exit 1
fi
[[ -x .githooks/pre-commit ]] || {
    echo 'error: .githooks/pre-commit must be executable' >&2
    exit 1
}
git config --local core.hooksPath .githooks
echo "Installed this repository's pre-commit formatter (.githooks)."

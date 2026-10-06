#!/usr/bin/env bash
set -euo pipefail

# Consumer fixture entry point for the immutable shared logger regression.
# Reuse actual Cargo receipt/index/retained-log cases, not Shared Tooling's
# changelog-only release adapter. Their child reset owns Make/release context.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
bash "$ROOT/scripts/release/test-release.sh" --metadata-only

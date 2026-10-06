#!/usr/bin/env bash
set -euo pipefail

# Consumer-owned Markdown selection; parsing belongs to the shared checker.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$ROOT"
roster="$(mktemp "${TMPDIR:-/tmp}/ic-backup-documents.XXXXXX")"
trap 'rm -f "$roster"' EXIT
git ls-files --cached --others --exclude-standard -z -- '*.md' > "$roster"
documents=()
count=0
while IFS= read -r -d '' document; do
    documents[count]="$document"
    count=$((count + 1))
done < "$roster"
[[ "$count" -gt 0 ]] || { echo 'no maintained Markdown documents found' >&2; exit 2; }
perl scripts/ci/check-documentation-links.pl --root "$ROOT" "${documents[@]}"

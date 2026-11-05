#!/usr/bin/env bash
set -euo pipefail

# Consumer-owned Markdown selection; parsing belongs to the shared checker.
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
cd "$ROOT"
scratch="${TMPDIR:-$ROOT/target/tmp}"
mkdir -p -- "$scratch"
fixture="$(mktemp -d "$scratch/ic-backup-documents.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
roster="$fixture/roster"
git ls-files --cached --others --exclude-standard -z -- '*.md' > "$roster"
# Keep pending Git deletions out of the working-source selection. Do not skip
# present but unreadable files or dangling links; the shared checker admits them.
git ls-files --deleted -z -- '*.md' > "$fixture/deleted"
perl -0e '
    open my $deleted, "<", $ARGV[0] or die "read deleted roster: $!";
    my %deleted = map { $_ => 1 } <$deleted>;
    close $deleted or die "close deleted roster: $!";
    open my $roster, "<", $ARGV[1] or die "read document roster: $!";
    while (<$roster>) {
        next if $deleted{$_};
        print or die "write selected roster: $!";
    }
    close $roster or die "close document roster: $!";
    close STDOUT or die "close selected roster: $!";
' "$fixture/deleted" "$roster" > "$fixture/selected"
documents=()
count=0
while IFS= read -r -d '' document; do
    documents[count]="$document"
    count=$((count + 1))
done < "$fixture/selected"
[[ "$count" -gt 0 ]] || { echo 'no maintained Markdown documents found' >&2; exit 2; }
perl scripts/ci/check-documentation-links.pl --root "$ROOT" "${documents[@]}"

#!/usr/bin/env bash
set -euo pipefail
# Nested publication validation must not write synthetic events into its parent's
# attempt journal. Each fixture owns its own publication directory below.
unset RELEASE_TIMING_JOURNAL RELEASE_TIMING_PARENT
# Exercise Backup's actual Make declaration and shared caller. Only the expensive
# validation command and Cargo publication effect are substituted, never uploaded.
root="$(git rev-parse --show-toplevel)"
real_cargo="$(command -v cargo)"
real_make="$(command -v make)"
mkdir -p "$root/target/publication-fixtures"
fixture="$(mktemp -d "$root/target/publication-fixtures/caller.XXXXXX")"
printf 'Publication caller fixture: %s\n' "$fixture"
git -C "$root" ls-files --cached --others --exclude-standard -z > "$fixture/paths"
while IFS= read -r -d '' path; do
    [[ -f "$root/$path" && ! -L "$root/$path" ]]
    mkdir -p "$fixture/$(dirname "$path")"
    cp -p "$root/$path" "$fixture/$path"
done < "$fixture/paths"
mv "$fixture/paths" "$fixture/source-paths"
cd "$fixture"
# Start from private eligibility even after a future authorized product release.
perl -0pi -e 's/^publish = .*$/publish = false/m or die "missing workspace eligibility"' Cargo.toml
git init --quiet
git config user.name 'Publication fixture'
git config user.email 'fixture@example.invalid'
git add -- .
git commit --quiet -m fixture
mkdir -p target/bin
cat > target/bin/cargo <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" != publish ]]; then exec "$PUBLICATION_REAL_CARGO" "$@"; fi
printf '%s\n' "$*" >> target/effects
[[ "$*" == "publish --locked --registry crates-io --package ic-backup@$PUBLICATION_VERSION --dry-run" ]]
exit "${PUBLICATION_COMMAND_STATUS:-0}"
CARGO
cat > target/bin/verify <<'VERIFY'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" != '--no-print-directory publication-verify' ]]; then
    exec "$PUBLICATION_REAL_MAKE" "$@"
fi
printf '%s\n' verified >> target/effects
case "${PUBLICATION_GATE:-pass}" in
    pass) ;;
    fail) exit 19 ;;
    changed) git commit --quiet --allow-empty -m 'Fixture concurrent source change' ;;
    *) exit 2 ;;
esac
VERIFY
chmod +x target/bin/cargo target/bin/verify
export PUBLICATION_REAL_CARGO="$real_cargo"
export PUBLICATION_REAL_MAKE="$real_make"
export PATH="$fixture/target/bin:$PATH"
publication_check() {
    make --no-print-directory "MAKE_COMMAND=$fixture/target/bin/verify" \
        "CARGO_PUBLISH_EVIDENCE_ROOT=$fixture/target/publication" publish-check "$@"
}
expect_failure() {
    if "$@" > target/refusal.log 2>&1; then
        echo 'expected publication refusal' >&2
        exit 1
    fi
}
: > target/effects
# Private product and qualification packages cannot pass publication admission.
expect_failure publication_check
test ! -s target/effects
expect_failure publication_check PACKAGE=ic-backup-consumer
test ! -s target/effects
# Eligibility is enabled only in this disposable fixture, never in the product.
perl -0pi -e 's/publish = false/publish = ["crates-io"]/ or die "missing private default"' Cargo.toml
git add -- Cargo.toml
git commit --quiet -m 'Fixture eligibility only'
PUBLICATION_VERSION="$("$real_cargo" metadata --locked --offline --no-deps --format-version 1 | jq -er '.packages[] | select(.name == "ic-backup") | .version')"
export PUBLICATION_VERSION
for selection in '' 'ic-backup ic-backup' 'ic-backup-consumer'; do
    expect_failure publication_check "PACKAGE=$selection"
    test ! -s target/effects
done
PUBLICATION_GATE=fail expect_failure publication_check
[[ "$(cat target/effects)" == verified ]]
: > target/effects
PUBLICATION_GATE=changed expect_failure publication_check
[[ "$(cat target/effects)" == verified ]]
git show HEAD:README.md > README.md
: > target/effects
# Environment cannot redirect the declared registry or widen the package roster.
CARGO_PUBLISH_REGISTRY=other CARGO_PUBLISH_PACKAGES=other publication_check PACKAGE=ic-backup > target/success.log 2>&1
[[ "$(cat target/effects)" == "$(printf 'verified\npublish --locked --registry crates-io --package ic-backup@%s --dry-run' "$PUBLICATION_VERSION")" ]]
: > target/effects
PUBLICATION_COMMAND_STATUS=23 expect_failure publication_check
[[ "$(wc -l < target/effects | tr -d ' ')" == 2 ]]
# A failed command is retained once and not retried or mistaken for success.
found=false
for status in target/publication/check.*/cargo-status; do
    [[ -f "$status" ]] || continue
    if [[ "$(cat "$status")" == 23 ]]; then found=true; fi
done
[[ "$found" == true && -z "$(git status --porcelain)" ]]
: > target/effects
mkdir target/publication/.cargo-publication.lock
expect_failure publication_check
[[ "$(cat target/effects)" == verified ]]
test -d target/publication/.cargo-publication.lock
printf 'Publication caller: eligibility, selection, gates, source drift and single-attempt failure PASS\n'

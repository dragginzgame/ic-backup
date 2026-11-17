#!/usr/bin/env bash
set -euo pipefail
# Caller qualification must not write synthetic events into a parent's attempt
# journal. Each fixture owns its own publication directory below.
unset RELEASE_TIMING_JOURNAL RELEASE_TIMING_PARENT
# Exercise Backup's actual Make declaration and shared caller. Only the expensive
# validation command and Cargo publication effect are substituted, never uploaded.
root="$(git rev-parse --show-toplevel)"
export PATH="$root/.tools/host/bin:$PATH"
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
case "$*" in
    "publish --locked --registry crates-io --package ic-backup@$PUBLICATION_VERSION --dry-run" | "publish --locked --registry crates-io --package ic-backup@$PUBLICATION_VERSION") ;;
    *) exit 2 ;;
esac
exit "${PUBLICATION_COMMAND_STATUS:-0}"
CARGO
cat > target/bin/verify <<'VERIFY'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    '--no-print-directory publication-preflight') stage=preflight ;;
    '--no-print-directory publication-verify') stage=qualified ;;
    *) exec "$PUBLICATION_REAL_MAKE" "$@" ;;
esac
printf '%s\n' "$stage" >> target/effects
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
publication() {
    local action="$1"
    shift
    make --jobs=2 --no-print-directory "MAKE_COMMAND=$fixture/target/bin/verify" \
        "CARGO_PUBLISH_EVIDENCE_ROOT=$fixture/target/publication" "$action" "$@"
}
publication_check() { publication publish-check "$@"; }
publication_upload() { publication publish "$@"; }
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
# Direct upload needs no earlier check evidence and always qualifies the product.
publication_upload > target/direct-upload.log 2>&1
[[ "$(cat target/effects)" == "$(printf 'qualified\npublish --locked --registry crates-io --package ic-backup@%s' "$PUBLICATION_VERSION")" ]]
: > target/effects
PUBLICATION_GATE=fail expect_failure publication_check
[[ "$(cat target/effects)" == preflight ]]
: > target/effects
PUBLICATION_GATE=changed expect_failure publication_check
[[ "$(cat target/effects)" == preflight ]]
git show HEAD:README.md > README.md
: > target/effects
# Environment cannot redirect the declared registry or widen the package roster.
CARGO_PUBLISH_REGISTRY=other CARGO_PUBLISH_PACKAGES=other publication_check PACKAGE=ic-backup > target/success.log 2>&1
[[ "$(cat target/effects)" == "$(printf 'preflight\npublish --locked --registry crates-io --package ic-backup@%s --dry-run' "$PUBLICATION_VERSION")" ]]
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
# A prior dry-run is never an input proof: even missing/tampered records cannot
# bypass upload's full gate. Cargo publication still executes afresh each time.
: > target/effects
for status in target/publication/check.*/status; do printf 'tampered\n' > "$status"; done
CARGO_PUBLISH_VERIFY_TARGETS=publication-preflight publication_upload \
    CARGO_PUBLISH_VERIFY_TARGETS=publication-preflight > target/upload.log 2>&1
[[ "$(cat target/effects)" == "$(printf 'qualified\npublish --locked --registry crates-io --package ic-backup@%s' "$PUBLICATION_VERSION")" ]]
: > target/effects
PUBLICATION_GATE=fail expect_failure publication_upload
[[ "$(cat target/effects)" == qualified ]]
: > target/effects
PUBLICATION_GATE=changed expect_failure publication_upload
[[ "$(cat target/effects)" == qualified ]]
: > target/effects
printf '\nfixture dirty source\n' >> README.md
expect_failure publication_upload
[[ "$(cat target/effects)" == qualified ]]
git show HEAD:README.md > README.md
: > target/effects
PUBLICATION_COMMAND_STATUS=29 expect_failure publication_upload
[[ "$(wc -l < target/effects | tr -d ' ')" == 2 ]]
# Preserve the failed upload and original command status, without a retry.
found=false
for status in target/publication/publish.*/cargo-status; do
    [[ -f "$status" ]] || continue
    if [[ "$(cat "$status")" == 29 ]]; then found=true; fi
done
[[ "$found" == true ]]
# Check+upload runs exactly one full qualification, without a receipt/cache.
: > target/effects
start="$(perl -MTime::HiRes=time -e 'print time')"
publication_check > target/preflight-timed.log 2>&1
middle="$(perl -MTime::HiRes=time -e 'print time')"
publication_upload > target/upload-timed.log 2>&1
end="$(perl -MTime::HiRes=time -e 'print time')"
[[ "$(grep -c '^qualified$' target/effects)" == 1 ]]
[[ "$(grep -c '^preflight$' target/effects)" == 1 ]]
[[ "$(grep -c '^publish ' target/effects)" == 2 ]]
perl -e 'printf "Simulated warm preflight %.3fs, upload %.3fs; one full qualification; effects substituted.\n", $ARGV[1]-$ARGV[0], $ARGV[2]-$ARGV[1]' "$start" "$middle" "$end"
: > target/effects
mkdir target/publication/.cargo-publication.lock
expect_failure publication_check
[[ "$(cat target/effects)" == preflight ]]
test -d target/publication/.cargo-publication.lock
printf 'Publication caller: preflight/full-gate selection, fresh dispatch, drift and failure custody PASS\n'

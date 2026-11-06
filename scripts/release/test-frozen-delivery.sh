#!/usr/bin/env bash
set -euo pipefail

# Actual Backup sources, Make hooks, editor, setup, locked native build, archives
# and Git transactions. Only the upload and a lost commit reply are substituted.
# All repositories/remotes/effects are disposable fixtures beneath our target.
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && pwd -P)"
unset RELEASE_SOURCE RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_COMMIT RELEASE_GROUP
unset RELEASE_REMOTE RELEASE_BRANCH RELEASE_MAKE RELEASE_DELIVERY RELEASE_KIND VERSION
unset RELEASE_GROUPS RELEASE_PUBLISH_TARGET RELEASE_SETUP_TARGET
unset RELEASE_SETUP_MODE RELEASE_BATCH_ACTIVE BATCH PUBLICATION_RESULT
unset SHARED_RELEASE_BATCH SHARED_PUBLICATION_RESULT SHARED_TOOLING_ROOT
unset MAKEFLAGS MFLAGS MAKEOVERRIDES MAKELEVEL GNUMAKEFLAGS MAKEFILES CARGO_TARGET_DIR
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH
export CARGO_NET_OFFLINE=true
FROZEN_REAL_GIT="$(command -v git)"
FROZEN_REAL_MAKE="$(command -v make)"
FROZEN_REAL_CARGO="$(command -v cargo)"
export FROZEN_REAL_GIT FROZEN_REAL_MAKE FROZEN_REAL_CARGO
mkdir -p "$ROOT/target"
fixture="$(mktemp -d "$ROOT/target/frozen-delivery-tests.XXXXXX")"
export FROZEN_FIXTURE="$fixture/consumer" FROZEN_CONTROL="$fixture/control"
mkdir -p "$FROZEN_FIXTURE" "$FROZEN_CONTROL" "$fixture/bin"
complete=false
finish() {
    local status=$?
    [[ "$complete" == true || "$status" != 0 ]] || status=1
    printf 'Frozen Backup qualification retained: %s (status %s)\n' "$fixture" "$status"
    exit "$status"
}
trap finish EXIT
# Copy working inputs, including this uncommitted adoption, never user Git state,
# tool stores or ignored evidence. This copy is only a reproducible test fixture.
git -C "$ROOT" ls-files --cached --others --exclude-standard -z > "$FROZEN_CONTROL/inventory"
while IFS= read -r -d '' path; do
    if [[ -f "$ROOT/$path" && ! -L "$ROOT/$path" ]]; then
        printf '%s\0' "$path"
    elif [[ -e "$ROOT/$path" || -L "$ROOT/$path" ]]; then
        echo "fixture source is not a regular file: $path" >&2; exit 1
    fi
done < "$FROZEN_CONTROL/inventory" > "$FROZEN_CONTROL/files"
tar -C "$ROOT" --null -T "$FROZEN_CONTROL/files" -cf "$FROZEN_CONTROL/source.tar"
tar -C "$FROZEN_FIXTURE" -xf "$FROZEN_CONTROL/source.tar"
cat >> "$FROZEN_FIXTURE/Makefile" <<'MAKE'
# Fixture qualification selects real native/source/archive leaves, not recursive
# tooling/release fixtures or a second full product simulator suite.
CI_TARGETS := compiler-check shared-tooling-check tools-check dependency-pins-check dependency-boundaries-check candid-boundary-check fmt-check check package
MAKE
git init -q --bare "$fixture/remote.git"
git -C "$FROZEN_FIXTURE" init -q -b main
git -C "$FROZEN_FIXTURE" config user.name 'Backup frozen fixture'
git -C "$FROZEN_FIXTURE" config user.email 'backup-fixture@example.invalid'
git -C "$FROZEN_FIXTURE" add .
git -C "$FROZEN_FIXTURE" commit -qm 'Reproducible Backup fixture source'
git -C "$FROZEN_FIXTURE" remote add fixture "$fixture/remote.git"
git -C "$FROZEN_FIXTURE" push -q fixture main
base="$(git -C "$FROZEN_FIXTURE" rev-parse HEAD)"
previous="$(cd "$FROZEN_FIXTURE" && perl scripts/release/release-data.pl version)"
version="$(bash "$ROOT/scripts/ci/next-release-version.sh" "$previous" patch)"

cat > "$fixture/bin/make" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s|%s\n' "$PWD" "$*" >> "$FROZEN_CONTROL/make.log"
exec "$FROZEN_REAL_MAKE" "$@"
SH
cat > "$fixture/bin/git" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$PWD" == "$FROZEN_FIXTURE"/target/release-frozen/* && "${1:-}" == commit && ! -e "$FROZEN_CONTROL/lost-commit" ]]; then
    "$FROZEN_REAL_GIT" "$@"
    touch "$FROZEN_CONTROL/lost-commit"
    printf '\n# Deferred normal-checkout update\n' >> "$FROZEN_FIXTURE/Cargo.lock"
    exit 9
fi
exec "$FROZEN_REAL_GIT" "$@"
SH
cat > "$fixture/bin/cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" != publish ]]; then exec "$FROZEN_REAL_CARGO" "$@"; fi
version="$(perl scripts/release/release-data.pl version)"
[[ "$*" == "publish --locked --registry crates-io --package ic-backup@$version --package ic-backup-agent@$version" ]]
[[ "${RELEASE_GROUP:-}" == backup && "${RELEASE_COMMIT:-}" == "$(git rev-parse HEAD)" ]]
printf '%s\n' "$PWD" "$RELEASE_COMMIT" >> "$FROZEN_CONTROL/uploads"
# Cargo performs real package/archive verification; no registry upload is run.
"$FROZEN_REAL_CARGO" package --offline --locked --workspace
for package in ic-backup ic-backup-agent; do
    archive="target/package/$package-$version.crate"
    tar -xzOf "$archive" "$package-$version/.cargo_vcs_info.json" > "$FROZEN_CONTROL/$package-vcs.json"
    jq -e --arg commit "$RELEASE_COMMIT" '.git.sha1 == $commit and (.git.dirty // false) == false' "$FROZEN_CONTROL/$package-vcs.json" >/dev/null
    bash scripts/ci/verify-file-checksum.sh --print sha256 "$archive" >> "$FROZEN_CONTROL/archive-checksums"
done
printf 'simulated registry accepted both exact archives\n' > "$FROZEN_CONTROL/registry-receipt"
exit 7
SH
chmod +x "$fixture/bin/make" "$fixture/bin/git" "$fixture/bin/cargo"
export PATH="$fixture/bin:$ROOT/.tools/host/bin:$PATH"
cd "$FROZEN_FIXTURE"
refuse() {
    if "$@"; then echo "unexpected frozen acceptance: $*" >&2; exit 1; fi
}
# The actual default frozen entrypoint, with a non-default saved remote and an
# explicitly selected publication hook. Initial setup and native/archive checks
# run once; a completed real candidate commit then loses its reply.
refuse make --no-print-directory release-patch RELEASE_REMOTE=fixture RELEASE_PUBLISH_TARGET=publish
intents=(.git/release-state/frozen/batch-*/intent.json)
[[ "${#intents[@]}" == 1 && -f "${intents[0]}" && -e "$FROZEN_CONTROL/lost-commit" ]]
id="$(basename "${intents[0]%/*}")"
work="$FROZEN_FIXTURE/target/release-frozen/$id"
state="$FROZEN_FIXTURE/.git/release-state/frozen/$id"
[[ "$(git rev-parse HEAD)" == "$base" ]]
grep -F 'Deferred normal-checkout update' Cargo.lock >/dev/null
cp Cargo.lock "$FROZEN_CONTROL/deferred.lock"
[[ -f "$state/repository.git/release-state/backup-$version.plan" ]]
jq -e --arg base "$base" '.schema == 2 and .phase == "releasing" and .source == $base and .groups == ["backup"] and .remote == "fixture" and .branch == "main"' "${intents[0]}" >/dev/null
refuse make --no-print-directory release-batch-retire "BATCH=$id"
[[ -d "$work" && -f "${intents[0]}" ]]

# Resume uses the exact original plan and offline tool checks; the real receipt,
# tag and validation sidecar are consumed before the simulated uncertain upload.
refuse make --no-print-directory release-batch-resume "BATCH=$id"
jq -e '.phase == "publishing"' "${intents[0]}" >/dev/null
[[ -f "$FROZEN_CONTROL/registry-receipt" && "$(wc -l < "$FROZEN_CONTROL/uploads")" -eq 2 ]]
delivered="$(git --git-dir="$fixture/remote.git" rev-parse refs/heads/main)"
jq -e --arg delivered "$delivered" '.schema == 2 and .release_commit == $delivered' "${intents[0]}" >/dev/null
[[ "$(git --git-dir="$fixture/remote.git" rev-parse "refs/tags/backup-v$version^{}")" == "$delivered" ]]
[[ "$(git --git-dir="$state/repository.git" log -1 --format=%P "$delivered")" == "$base" ]]
grep -F '|--no-print-directory install-tools' "$FROZEN_CONTROL/make.log" > "$FROZEN_CONTROL/installations"
[[ "$(wc -l < "$FROZEN_CONTROL/installations")" -eq 1 ]]
cmp Cargo.lock "$FROZEN_CONTROL/deferred.lock"
refuse make --no-print-directory release-batch-resume "BATCH=$id"
[[ "$(wc -l < "$FROZEN_CONTROL/uploads")" -eq 2 ]]

# The explicit fixture registry receipt above resolves publication. Completed
# replay performs no setup, release or second publication.
make --no-print-directory release-batch-resume "BATCH=$id" PUBLICATION_RESULT=published
make --no-print-directory release-batch-resume "BATCH=$id"
[[ "$(wc -l < "$FROZEN_CONTROL/uploads")" -eq 2 ]]
cmp Cargo.lock "$FROZEN_CONTROL/deferred.lock"
jq -e --arg delivered "$delivered" '.phase == "complete" and .commit == $delivered' "${intents[0]}" >/dev/null
make --no-print-directory release-batch-status "BATCH=$id"
refuse make --no-print-directory release-batch-reconcile "BATCH=$id"
cmp Cargo.lock "$FROZEN_CONTROL/deferred.lock"

# Preserve deferred work on a fixture branch before the explicit fast-forward.
git checkout -qb deferred
git add Cargo.lock
git commit -qm 'Preserve fixture deferred update'
git checkout -q main
make --no-print-directory release-batch-reconcile "BATCH=$id"
[[ "$(git rev-parse HEAD)" == "$delivered" ]]
git show deferred:Cargo.lock > "$FROZEN_CONTROL/preserved.lock"
cmp "$FROZEN_CONTROL/preserved.lock" "$FROZEN_CONTROL/deferred.lock"
mkdir "$FROZEN_CONTROL/retained-release-state"
cp -R "$state/repository.git/release-state/." "$FROZEN_CONTROL/retained-release-state/"
cp "${intents[0]}" "$FROZEN_CONTROL/completed-intent.json"
make --no-print-directory release-batch-retire "BATCH=$id"
[[ ! -e "$work" && ! -e "$state" ]]
[[ "$(git rev-parse "refs/tags/backup-v$version^{}")" == "$delivered" ]]
complete=true
echo 'Frozen Backup delivery: PASS (actual setup/build/archives/receipts, deferred lock custody, exact recovery, uncertain publication, reconciliation and explicit retirement).'

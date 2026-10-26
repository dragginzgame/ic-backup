#!/usr/bin/env bash
set -euo pipefail

# Isolated fixtures own their logging context, including fallback TMPDIR paths.
# Do not inherit an outer validation runner's retained-log selection or identity.
unset VALIDATION_LOG_DIR VALIDATION_FAILURE_LOG_DIR VALIDATION_FAILURE_EVENT_PREFIX
unset VALIDATION_RUNNER_DEPTH VALIDATION_RUNNER_SNAPSHOT_PATH VALIDATION_REPOSITORY_ROOT

# Consumer integration checks; no Git mutations, network calls or Rust builds.
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
mkdir -p "$ROOT/target"
FIXTURE="$(mktemp -d "$ROOT/target/shared-tooling-tests.XXXXXX")"
CONSUMER="$FIXTURE/consumer"
fixture_complete=false

finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" -eq 0 ]]; then
        rm -rf "$FIXTURE"
    else
        printf 'Shared-tooling tests failed; fixture/logs retained: %s\n' "$FIXTURE" >&2
    fi
    exit "$status"
}
trap finish EXIT

# Exercise this consumer's actual CI recipe, without compilation or duplicate
# recipes. Only these unique targets substitute the graph-edit/next-stage effects.
checkpoint="$FIXTURE/input-checkpoint"
mkdir -p "$checkpoint/make" "$checkpoint/scripts/ci"
bash "$ROOT/scripts/ci/prepare-script-fixture.sh" "$ROOT" "$checkpoint" \
    Makefile make/consumer.mk scripts/ci/run-validation-targets.sh
cat >> "$checkpoint/Makefile" <<'MAKE'
.PHONY: fixture-mutate-input fixture-later-stage
fixture-mutate-input:
	@printf 'changed\n' >> "$(FIXTURE_CHANGED_INPUT)"
fixture-later-stage:
	@printf 'dispatched\n' >> later-stage
MAKE
for changed in Cargo.toml Cargo.lock; do
    printf 'original manifest\n' > "$checkpoint/Cargo.toml"
    printf 'original lock\n' > "$checkpoint/Cargo.lock"
    if env -u MAKEFLAGS -u MFLAGS -u MAKELEVEL -u MAKEFILES \
        make --no-print-directory -C "$checkpoint" ci \
        'CI_TARGETS=fixture-mutate-input fixture-later-stage' "FIXTURE_CHANGED_INPUT=$changed" \
        > "$checkpoint/$changed.log" 2>&1; then
        echo 'consumer CI accepted changed inputs' >&2; exit 1
    fi
    [[ ! -e "$checkpoint/later-stage" ]] || exit 1
    grep -F "validation input changed: $changed" "$checkpoint/$changed.log" >/dev/null || exit 1
    grep -F 'Restart preparation and the full gate' "$checkpoint/$changed.log" >/dev/null || exit 1
    [[ -s "$checkpoint/target/validation-failures/latest-combined.log" ]] || exit 1
    if grep -E 'overriding recipe|ignoring old recipe' "$checkpoint/$changed.log"; then exit 1; fi
done
echo 'Consumer manifest/lock checkpoints: PASS (changed inputs stop later CI dispatch).'

# Exercise the actual local Make boundary target with both common and local policy.
# Metadata is substituted only inside this tooling fixture; production uses Cargo.
boundary="$FIXTURE/boundary"
mkdir -p "$boundary/ci" "$boundary/make" "$boundary/scripts/ci" "$boundary/bin"
bash "$ROOT/scripts/ci/prepare-script-fixture.sh" "$ROOT" "$boundary" \
    Makefile make/consumer.mk scripts/ci/check-cargo-dependency-boundaries.sh \
    ci/external-ic-dependencies.txt ci/cargo-dependency-boundaries.json
printf '[package]\nname = "boundary-fixture"\nversion = "0.0.0"\n' > "$boundary/Cargo.toml"
cat > "$boundary/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
cat "$BOUNDARY_METADATA"
CARGO
chmod +x "$boundary/bin/cargo"
export BOUNDARY_METADATA="$boundary/metadata.json"
for dependency in ic-platform ic0 wat; do
    for kind in normal build dev; do
        jq -n --arg name "$dependency" --arg kind "$kind" --arg root "$boundary" '{
            version:1,workspace_root:$root,workspace_members:["fixture"],
            packages:[{id:"fixture",name:"boundary-fixture",dependencies:[{
                name:$name,rename:"renamed_dependency",kind:(if $kind == "normal" then null else $kind end),
                target:"cfg(unix)",optional:true,features:["fixture-feature"]
            }]}]
        }' > "$BOUNDARY_METADATA"
        status=0
        PATH="$boundary/bin:$PATH" make --no-print-directory -s -C "$boundary" \
            dependency-boundaries-check > "$boundary/$dependency-$kind.log" 2>&1 || status=$?
        if [[ "$dependency" == ic-platform ]]; then
            [[ "$status" == 0 ]] || { cat "$boundary/$dependency-$kind.log" >&2; exit 1; }
        else
            [[ "$status" != 0 ]] || { echo "local gate admitted forbidden $kind dependency: $dependency" >&2; exit 1; }
            grep -F "boundary-fixture -> $dependency" "$boundary/$dependency-$kind.log" >/dev/null
        fi
    done
done
unset BOUNDARY_METADATA
echo 'Consumer dependency boundaries: PASS (common IC and local WAT rules compose through Make).'

# Independent consumers contain only their selected production graph. Check
# package identities, including a renamed transitive test/tool dependency.
graph="$FIXTURE/graph"
mkdir "$graph"
jq -n '{packages: [{name:"runtime", version:"1.0.0", source:"registry"}]}' > "$graph/original.json"
cp "$graph/original.json" "$graph/selected.json"
jq -L "$ROOT/scripts/ci" -e --slurpfile original "$graph/original.json" '
    include "consumer-graph"; original_graph_admitted($original[0]; "consumer")
' "$graph/selected.json" > /dev/null
for dependency in ic-testkit pocket-ic wat; do
    jq --arg name "$dependency" '
        .packages += [{name:$name, version:"1.0.0", source:"registry"}]
        | .resolve = {nodes:[{id:"runtime", deps:[{name:"renamed_tool", pkg:$name}]}]}
    ' "$graph/selected.json" > "$graph/forbidden.json"
    if jq -L "$ROOT/scripts/ci" -e --slurpfile original "$graph/forbidden.json" '
        include "consumer-graph"; original_graph_admitted($original[0]; "consumer")
    ' "$graph/forbidden.json" > /dev/null; then
        echo "consumer graph accepted test/tool package: $dependency" >&2; exit 1
    fi
done
echo 'Consumer production graph: PASS (simulator and WAT packages refuse by identity).'

# Published qualification admits only exact workspace-to-crates.io substitutions.
jq -n '{packages: [
    {name:"ic-backup", version:"0.41.0", source:null},
    {name:"ic-backup-agent", version:"0.41.0", source:null},
    {name:"runtime", version:"1.0.0", source:"registry+https://github.com/rust-lang/crates.io-index"}
]}' > "$graph/release-original.json"
jq '.packages |= map(.source = "registry+https://github.com/rust-lang/crates.io-index")' \
    "$graph/release-original.json" > "$graph/release-selected.json"
jq -L "$ROOT/scripts/ci" -e --slurpfile original "$graph/release-original.json" '
    include "consumer-graph"; published_graph_admitted($original[0]; "consumer")
' "$graph/release-selected.json" >/dev/null
for mutation in version path git runtime simulator missing; do
    jq --arg mutation "$mutation" '
        if $mutation == "version" then .packages[0].version = "0.42.0"
        elif $mutation == "path" then .packages[0].source = null
        elif $mutation == "git" then .packages[0].source = "git+https://example.invalid/backup"
        elif $mutation == "runtime" then .packages[2].version = "2.0.0"
        elif $mutation == "simulator" then .packages += [{name:"pocket-ic", version:"16.1.0", source:"registry+https://github.com/rust-lang/crates.io-index"}]
        else .packages |= map(select(.name != "ic-backup-agent")) end
    ' "$graph/release-selected.json" > "$graph/release-refused.json"
    if jq -L "$ROOT/scripts/ci" -e --slurpfile original "$graph/release-original.json" '
        include "consumer-graph"; published_graph_admitted($original[0]; "consumer")
    ' "$graph/release-refused.json" >/dev/null; then
        echo "published consumer graph accepted invalid selection: $mutation" >&2; exit 1
    fi
done
echo 'Published consumer graph: PASS (exact release sources only).'


# Contradict a successful local command result: Bash 3.2 must stop at the
# assertion rather than continue to completion and discard its original inputs.
assertion="$FIXTURE/assertion"
mkdir -p "$assertion/bin"
export ASSERTION_REAL_HEAD
ASSERTION_REAL_HEAD="$(command -v head)"
cat > "$assertion/bin/head" <<'HEAD'
#!/usr/bin/env bash
if [[ "$*" == '-n 1 '* ]]; then
    printf 'wrong first command\n'
else
    exec "$ASSERTION_REAL_HEAD" "$@"
fi
HEAD
chmod +x "$assertion/bin/head"
status=0
TMPDIR="$assertion" PATH="$assertion/bin:$PATH" "$BASH" "$ROOT/scripts/ci/test-testkit-commands.sh" \
    > "$assertion/output.log" 2>&1 || status=$?
[[ "$status" == 1 ]] || exit 1
retained="$(sed -n 's/^Testkit command fixture retained: //p' "$assertion/output.log")"
[[ -n "$retained" && -s "$retained/order.log" ]] || exit 1
if grep -F 'Testkit setup/check routing' "$assertion/output.log" > /dev/null; then exit 1; fi

# Qualify failure retention through the actual adopted scripts. Intercept only
# their first child helper after original fixture inputs have been written.
# Use the selected Bash directly; generating its shebang avoids BSD sed's
# replacement-newline behavior and an env-bash recursion through this wrapper.
retention="$FIXTURE/retention"
mkdir -p "$retention/bin"
{
    printf '#!%s\n' "$BASH"
    cat <<'BASH'
if [[ "${1:-}" == */scripts/ci/"$RETENTION_FAILED_HELPER" ]]; then
    # Admission fixtures must reach the real guard and their failing control
    # gate before the original retained patch fixture receives our injection.
    # The new goal-admission cases must reach the actual logger. Inject only
    # at the original passing-output case whose fixture retention we qualify.
    if [[ "$RETENTION_FAILED_HELPER" == run-validation-targets.sh && "${2:-}" != passing-tests ]]; then
        exec "$RETENTION_REAL_BASH" "$@"
    fi
    if [[ "$RETENTION_FAILED_HELPER" != check-dependency-pins.sh ]] &&
        ! "$RETENTION_REAL_BASH" "$RETENTION_EXECUTION_CHECK" "${RELEASE_MAKE:-make}" >/dev/null 2>&1; then
        exec "$RETENTION_REAL_BASH" "$@"
    fi
    printf 'injected child-helper status: %s\n' "$RETENTION_CHILD_STATUS" >&2
    exit "$RETENTION_CHILD_STATUS"
fi
exec "$RETENTION_REAL_BASH" "$@"
BASH
} > "$retention/bin/bash"
chmod +x "$retention/bin/bash"
for owner in dependency-pins validation-target-runner; do
    case "$owner" in
        dependency-pins)
            helper=check-dependency-pins.sh; child_status=23; expected_status=1
            prefix='dependency-pins-test'; original=consumer/Cargo.toml ;;
        validation-target-runner)
            helper=run-validation-targets.sh; child_status=31; expected_status=31
            prefix=validation-runner-test; original=Makefile ;;
    esac
    mkdir "$retention/$owner"
    status=0
    TMPDIR="$retention/$owner" PATH="$retention/bin:$PATH" \
        RETENTION_REAL_BASH="$BASH" RETENTION_EXECUTION_CHECK="$ROOT/scripts/ci/check-make-execution.sh" \
        RETENTION_FAILED_HELPER="$helper" \
        RETENTION_CHILD_STATUS="$child_status" \
        "$BASH" "$ROOT/scripts/ci/test-$owner.sh" > "$retention/$owner.log" 2>&1 || status=$?
    [[ "$status" == "$expected_status" ]] || {
        cat "$retention/$owner.log" >&2
        echo "unexpected $owner failure status: $status" >&2
        exit 1
    }
    retained=("$retention/$owner/$prefix".*)
    [[ "${#retained[@]}" == 1 && -f "${retained[0]}/$original" ]] || exit 1
    rg -F "${retained[0]}" "$retention/$owner.log" >/dev/null
    captured="$retention/$owner.log"
    case "$owner" in
        dependency-pins) rg -Fx 'version = "0.1.0"' "${retained[0]}/$original" >/dev/null ;;
        validation-target-runner)
            captured="${retained[0]}/passing-tests.log"
            rg -F 'first-failure-marker' "${retained[0]}/$original" >/dev/null ;;
    esac
    # A missing/broken launcher must not accidentally satisfy a refusal case.
    rg -Fx "injected child-helper status: $child_status" "$captured" >/dev/null
    printf 'Fixture retention: PASS [%s] (status %s; original input retained).\n' "$owner" "$status"
done

reset_consumer() {
    mkdir -p "$CONSUMER"
    # Shared documents may link the consumer's own README. It is a local input,
    # not a payload to import from Shared Tooling or add to its snapshot.
    cp "$ROOT/README.md" "$CONSUMER/README.md"
    cp "$ROOT/.shared-tooling.snapshot" "$CONSUMER/"
    while IFS=$'\t' read -r record _digest _mode path; do
        [[ "$record" == file ]] || continue
        mkdir -p "$CONSUMER/$(dirname "$path")"
        # Remove only this test's destination (which a prior case may symlink).
        rm -f "$CONSUMER/$path"
        cp -p "$ROOT/$path" "$CONSUMER/$path"
    done <"$ROOT/.shared-tooling.snapshot"
}

verify() {
    bash "$ROOT/scripts/ci/verify-shared-tooling-snapshot.sh" --consumer "$CONSUMER"
}

expect_rejection() {
    local status=0
    verify >"$FIXTURE/$1.log" 2>&1 || status=$?
    [[ "$status" -eq 1 ]] || {
        printf 'Expected snapshot rejection in %s; got %s\n' "$1" "$status" >&2
        exit 1
    }
    [[ "$(cat "$CONSUMER/target/evidence")" == 'retained evidence' ]] || exit 1
}

reset_consumer
mkdir -p "$CONSUMER/target"
printf 'retained evidence\n' >"$CONSUMER/target/evidence"
verify >"$FIXTURE/valid.log" 2>&1

# Qualify the bootstrap against this consumer's exact file set. The inspected
# helper must never execute, even when it would falsely approve every payload.
for changed in helper-only helper-and-payload; do
    printf '#!/usr/bin/env bash\nprintf "executed\\n" > %q\nexit 0\n' \
        "$CONSUMER/target/helper-executed" > "$CONSUMER/scripts/ci/verify-file-checksum.sh"
    chmod +x "$CONSUMER/scripts/ci/verify-file-checksum.sh"
    if [[ "$changed" == helper-and-payload ]]; then
        printf '\nchanged payload\n' >> "$CONSUMER/docs/principles/README.md"
    fi
    expect_rejection "$changed"
    [[ ! -e "$CONSUMER/target/helper-executed" ]] || exit 1
    reset_consumer
done

# The reviewed governance roster must be closed inside the exported consumer,
# without borrowing linked documents from the upstream checkout.
governance_documents=()
while IFS= read -r path || [[ -n "$path" ]]; do
    [[ -f "$CONSUMER/$path" && ! -L "$CONSUMER/$path" ]] || exit 1
    case "$path" in
        *.md) governance_documents[${#governance_documents[@]}]="$path" ;;
    esac
done < "$CONSUMER/scripts/distribution/governance-files.txt"
perl "$ROOT/scripts/ci/check-documentation-links.pl" --root "$CONSUMER" \
    "${governance_documents[@]}" > "$FIXTURE/governance-links.log" 2>&1

printf '\nchanged bytes\n' >>"$CONSUMER/docs/principles/README.md"
expect_rejection changed-bytes
reset_consumer

chmod +x "$CONSUMER/docs/principles/README.md"
expect_rejection changed-mode
reset_consumer

chmod -x "$CONSUMER/scripts/ci/run-validation-targets.sh"
expect_rejection lost-executable-mode
reset_consumer

rm "$CONSUMER/docs/principles/README.md"
expect_rejection missing-file
reset_consumer

rm "$CONSUMER/docs/principles/README.md"
ln -s "$ROOT/docs/principles/README.md" "$CONSUMER/docs/principles/README.md"
expect_rejection symlink
reset_consumer

sed -n '/^file/{p;q;}' "$ROOT/.shared-tooling.snapshot" >>"$CONSUMER/.shared-tooling.snapshot"
expect_rejection duplicate-path
reset_consumer

printf 'file\t%064d\t-\t../outside\n' 0 >>"$CONSUMER/.shared-tooling.snapshot"
expect_rejection escaping-path
reset_consumer
verify >"$FIXTURE/recovered.log" 2>&1

echo 'Shared-tooling consumer checks: PASS (exact bytes/modes, rejection and evidence retention).'
fixture_complete=true

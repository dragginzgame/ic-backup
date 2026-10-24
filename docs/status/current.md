# Current handoff — 2026-10-24

## Ownership and source

Released Backup is **0.44.0**, `7d48b94`. Core owns bounded records, guarded
persistence, reconciliation and original-plan coordination. Agent owns reserved
signed transport; applications own current authority, membership, release binding,
whole-unit fencing and external obligations. Host owns generic IO/process/hash,
Platform external IC identities/SDKs, and Testkit simulator assets/processes.
[Implementation contracts](../implementation-contracts.md) retain product obligations.

## Shared Tooling adoption

Pending **0.44.1** adopts committed Shared Tooling **0.26.14**,
`e14050bcc0c627f9c8a9064051699cac44fd7ab3`, matching observed remote main.
Canonical impact review/export verifies all 124 selected files; 21 payloads change,
two are added and none retire. [The review](../shared-tooling-review.json) records
provenance and scope; [the adoption guide](../shared-tooling.md) owns consumer use.
No vendored payload is hand-edited. Existing prepared tools pass offline admission
without setup or rebuild; the common tool catalogs remain unchanged.

The actual Make boundary target composes Shared's external-IC inventory with four
additional Backup exclusions. Matching fixtures exercise aliases, optional/target
metadata and normal/build/dev declarations. Local hook fixtures carry the complete
literal-catalog parser closure. Testkit distinguishes busy status 75 from missing
installation without invoking its server or suggesting setup. Make retains its
normal wrapper exit behavior. `tools-report` diagnoses all offline checks while
setup/release admission remains fail-fast.

The release fixture now admits clean manually committed candidate metadata as a
fresh base while retaining the earlier uncertain intent, backups and validation.
Failed fresh gates preserve original metadata/receipts; later success completes
only the new increment. Actual release effects remain separately authorized.

## IC Tooling and preserved work

Registry checks confirm **0.68.0** is still the latest published Host Artifacts,
FS, Process, Testkit and Build release. All are already selected; no dependency
update or lock generation is needed. The unpublished 0.68.1 draft is not adopted.
The incoming Platform **0.6.15** lock edit is preserved byte-for-byte, SHA-256
`7211adb61040e98ef8f31f68a266f95fc4f7b90bd2ff7f2936834349eedc99c4`.
Workspace package versions remain 0.44.0. No product Rust, wire/storage or lifecycle
behavior changes; no staging, commit, push, release, publication or sibling edit.

## Qualification and limits

Snapshot, direct boundary/inheritance, consumer tooling/graph/refusal, formatting,
hook/index/partial-stage, Testkit command routing, focused ShellCheck, offline tools,
aggregate tool report, Backup release-adapter and canonical shared runner fixtures
pass. Evidence is in `/tmp/backup-shared-02614-*.log`; original failed fixtures are
retained. Release evidence: `target/shared-release-tests.DQHNf2/`; hook evidence:
`target/hook-tests.eMbaZ6/`. Documentation links and repository-file/license checks
are verified after this handoff update.

No new Rust/MSRV/package/simulator qualification is claimed for incoming Platform
0.6.15. Earlier product results retain their source/graph scope in the
[released handoff](https://github.com/dragginzgame/ic-backup/blob/7d48b94/docs/status/current.md).
Full local make ci, native macOS, installed Canic/IC and live effects are not run.
Direct declarations and independent consumer graph exclusions do not bind actual
deployed Wasm bytes; the deploying application owns that target/feature/artifact
gate. Canic still owns production integration and eventual retirement of its old
restore engine after replacement acceptance.

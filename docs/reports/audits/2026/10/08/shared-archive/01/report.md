# Archive retained consumer CI evidence

The compatible 0.7.1 batch adopts committed Shared Tooling 0.1.25
`eeb72e741199bd8574280eacb3542d8379b912f6`, with explicit canonical evidence helper
and regression additions. All 77 source blobs/modes match. [Snapshot proof](snapshot-proof.json)
records clean isolated consumer export and exact prior-manifest admission before
canonical output propagation into the still-uncommitted consumer adoption.
No root Git/index writes or vendored-source patches run. Common rules remain unchanged.

Exact-source [upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37762726615)
passes Linux, Intel macOS, Apple Silicon macOS and lint/security, including hosted
failure evidence archive/upload/download verification. Historical 0.1.24 evidence
and its failed hosted observation remain [preserved](previous-shared-0124-review.json).

The consumer workflow selects only its existing explicit fixture/failure roots and
archives them before the same v4 uploader. Artifact name and seven-day retention
remain unchanged. Hidden files, unusual names, 0640/executable modes and symlinks
stay inside the archive; final links are not followed and Git metadata is excluded.
The canonical archive test joins normal tooling and macOS system Bash qualification.
No new generic collector, product flow or local CLI is created.

[Actual collector proof](consumer-archive-proof.json) runs the workflow's parsed
inline body with controlled absent roots and early/late retained failures, then
extracts real tar archives and checks original bytes/status, names, modes and links.
Actionlint, canonical archive regression, ShellCheck and exact snapshot checks pass
locally on Linux. [Qualification](qualification.json) binds logs and inputs.
These are local real archive operations plus wiring proof, not a hosted consumer
upload/download or current native macOS result. [Backup #28](https://github.com/dragginzgame/ic-backup/issues/28)
therefore remains open for committed consumer acceptance. Upstream success cannot
substitute for the changed consumer workflow.

The ref-type race remains [shared #62](https://github.com/dragginzgame/shared-tooling/issues/62).
No release, publication, root Git writes, sibling edits or evidence cleanup run.
No functions, methods or types are removed; the prior direct-upload path is retired.

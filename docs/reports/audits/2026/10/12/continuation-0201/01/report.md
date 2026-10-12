# Shared Tooling 0.6.6 and Host 0.14.5 qualification

The released Backup base is 0.20.0 at
`2e9dbc4ccaae7822d07afcca40e39c1e1ae9c4b2`. Package versions remain 0.20.0;
the compatible draft is 0.20.1. The incoming lockfile already selected Host
0.14.5. No root manifest, index, ref or release action is part of this review.

Canonically export committed Shared Tooling 0.6.6
`a15eb9d406d287697df9bc8fa96db8cda6721a71` from a clean isolated source.
Three selected paths change and the 102-file roster remains intact. The common
`cargo-sort-derives` pin moves from 0.13.0 to 0.14.1. Explicit setup and offline
admission pass. A direct and conditional derive probe preserves both literal
`$crate` paths and is stable on a repeated run. Backup has no affected product
macros or default invocation of this formatter. Hook, formatting and consumer
tooling fixtures pass. This local probe does not replace the Shared Tooling
owner's separate native macro-consumer qualification.

All four Host 0.14.5 archives match their incoming lockfile checksums and
committed producer source `bb8f1e71f3d4599618d0f4084d27fa637408f1fb`.
Their Rust files are byte-identical to 0.14.4, and normalized published
manifests differ only in package version. The root Host requirements remain
`0.14`; no public identity cut, product offload or behavior change follows.
Registry fetch succeeded under the locked graph. The unchanged incoming lockfile
also selected Testkit 0.34.2 and Metrics 0.7.1. Metrics library Rust is unchanged
from 0.7.0; Testkit's 0.34.2 Rust change is confined to CLI store-path admission
and its test. No Testkit or Metrics change was made in this batch.

Every registered `make ci` target passes on the final unchanged source and lock
inputs, including strict Clippy, both real simulator suites, Rust 1.88 independent
consumers and paired package archives. The first gate stopped on the report link
before the report existed; after that correction, sandboxed PocketIC could not
bind loopback. A host package update then changed Cargo tool build identities
between fixture checks. The selected common and Testkit tools were explicitly
prepared again under the settled inventory, passed offline admission, and the
complete loopback-enabled gate passed. Failed runs and fixtures remain retained;
they are not represented as passing evidence. The root manifest, incoming lock,
technical files, real index and HEAD stayed unchanged across the final gate.
[Compact qualification evidence](qualification.json) records the exact graph,
targets and source identities; detailed local logs stay in ignored
`target/continuation-0201/`.

Shared Tooling 0.6.6 and Host 0.14.5 producer CI both pass Linux and native Intel/
Apple Silicon macOS at their exact commits. Released Backup 0.20.0 passes Linux;
both macOS jobs remain queued at final inspection. This uncommitted candidate has
no hosted native result, so Backup's exact native acceptance remains open.
Earlier reports retain their original source and graph scopes.

Coordination: [Backup #36](https://github.com/dragginzgame/ic-backup/issues/36),
[Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113).

# Shared Tooling 0.6.6 and Host 0.14.4 inspection

The current qualified working-tree snapshot remains Shared Tooling 0.6.5 with
Host 0.14.3, Testkit 0.34.1 and Metrics 0.7.0. Pending notes remain 0.20.0;
released Backup is 0.19.2. This request inspects new releases and applies no
snapshot, pin, lockfile or implementation update.

Shared Tooling remote main selects committed 0.6.6
`a15eb9d406d287697df9bc8fa96db8cda6721a71`. Clean canonical preview changes
three selected files: the common tool catalog, local setup guide and hook rules.
The 102-file roster stays unchanged. It selects cargo-sort-derives 0.14.1 rather
than 0.13.0, fixing literal `$crate` replacement in direct/conditional derives.
The formatter build floor is Rust 1.88; Backup's supported library floor and
prepared developer compiler already satisfy their independent requirements.
Adoption must explicitly prepare/admit the new common Rust selection; retain
previous bundles and evidence. This is contributor tooling, with no new library
API cut or persisted backup record conversion.

Source tracing finds no `$crate` macro definition, protection marker or
cargo-sort-derives call in Backup's product/default formatter inputs. The canonical
workspace formatting command still runs Cargo manifest sorting and rustfmt.
No local macro rewrite or removal of unrelated sorting exclusions is needed.
The shared real-hook macro qualification remains upstream-owned, separate from
consumer setup/formatting admission. The shared frontend fixture is not a new
Backup validation requirement.

All four Host packages are published at 0.14.4, non-yanked, with Rust 1.88.
Their downloaded archives match the official registry-index checksums and
published committed source `f93cd8854d500616a73eda713d215a20ff0617c7`, verified
locally and on GitHub. At the initial remote-main observation Host still showed
0.14.3; the published 0.14.4 release commit is now independently verified rather
than inferred from a local draft label. Published Rust is identical to 0.14.3;
normalized package manifests differ only by version. The release adopts Shared
0.6.6 contributor tooling. No new Backup API/offload or Host type cut is demonstrated.

Exact Shared 0.6.6 CI passes Linux and lint/security; Intel macOS is running and
Apple Silicon queued at the bounded inspection. This does not establish complete
native acceptance. No full Backup suite, tool setup, production IC, real Git/index,
version/release operation or sibling edit ran. Existing source/graph/index/HEAD
are unchanged; the prior full-gate result is not relabelled as 0.6.6/0.14.4 proof.

Evidence stays under ignored `target/shared-066-review/`; [compact source evidence](qualification.json)
binds checksums, published identities and the preview. Recommended adoption remains
under [Backup #36](https://github.com/dragginzgame/ic-backup/issues/36), with formatter
repair/consumer qualification owned by [Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113).

# Shared Tooling 0.6.2 adoption

Adopt committed `7f1ca16b0dfa98021d2671017ecb7de767409b0e` through clean canonical
preview/export after it landed during 0.6.1 qualification. The 99-file roster stays
intact; two guides and the release runner/fixture change from qualified 0.6.1.
Backup remains released 0.18.3 with compatible pending 0.18.4 notes and retained
fence-observation work.

Interrupted release source conflicts retain exact plan, phase, version and commit
diagnostics. Recovery requires maintainer authority and positive consumer-specific
proof that preparation never began; original plans/validation remain retained.
No recovery or release transaction is executed in this adoption. Audit/Machete
guidance distinguishes standalone arguments and inherited Cargo launcher context;
Backup has no extension caller to change.

Reuse the [passing 0.6.1 gate](../../shared-061/01/report.md): product sources,
locked graph, common installer/storage, native reuse and archive collection remain
unchanged. All five affected snapshot, documentation, shell, tooling and release
checks pass. Host 0.13.5 arrived during that phase; its Rust sources match 0.13.4.
All nine graph-dependent checks pass again on frozen final inputs, including
real simulator suites, strict Clippy, Rust 1.88 consumers and paired packages.
The three phases cover all 16 delivery targets. Tooling sources stayed fixed
through the lock refresh; final source/graph, real index and HEAD remain unchanged.
Released 0.18.3 passes Linux but both macOS jobs fail in the old Rust-tool fixture
immediately after first common setup. The adopted numeric-count fix covers this
boundary; matching candidate native macOS acceptance remains required. Cancelled
upstream 0.6.1 macOS jobs supply none. See [the compact proof](qualification.json).
Detailed phase evidence stays under ignored `target/shared-next`.

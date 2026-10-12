# Metrics 0.7 consumer adoption

Released Backup 0.19.2 is `2ebf8a2e6ea25061bc9a04a2ecca28ea5b910e54`.
Preserve its package versions and the incoming Metrics 0.7.0 manifest/lock selection.
The registry archive checksum matches the lock; every published source file matches
committed Metrics `384046884365e3d73696e0813a43640d3b1f2cb4`.

Public summary/aggregate totals and integer means now use u128. Samples, counts,
latest/maximum values and histogram bounds/buckets remain u64. The existing shared
arithmetic and sole prepared-byte histogram remain their owners; no local shim,
second accumulator or narrowing cast is added. Select pending 0.20.0 because the
re-exported Rust types change. Callers must align their direct Metrics dependency
and accumulated report fields. Unit conversion still excludes unrepresentable u64
samples independently, including successful bytes with an unrepresentable duration.

Diagnostics remain guard-local, empty on reopen and unpersisted. There is no Metrics
codec, generated product report, retained installation or journal conversion here.
Existing record identities, allowances, pending effects and application obligations
remain unchanged. No automatic logging task, alert window or new overflow owner is
required; callers inspect the shared flags when interpreting a diagnostic snapshot.

Focused recording cases pass, including wide duration totals in every success/failure
lane, wide byte totals with unchanged distribution, valid zero/repeated work and
unrepresentable sample exclusion. Public upload evidence and independent Rust 1.88
caller compilation exercise the re-exported wide types. Every registered full `make ci` target passes on unchanged final inputs, including
both actual simulator suites, strict Clippy, Rust 1.88 independent consumers and
paired package archives. [Compact qualification evidence](qualification.json)
binds registered gate membership and retained logs to their actual inputs.

Testkit changed externally from 0.34.0 to 0.34.1 during preparation. The first gate
refused its missing CLI; original failure evidence stays retained. Its only Rust
change preserves a failed download's primary error when writing stderr evidence
also fails; library Rust is unchanged. Explicit CLI setup and offline admission
precede the final frozen Host 0.14.2/Testkit 0.34.1/Metrics 0.7.0 graph. Earlier
focused evidence remains bound to Testkit 0.34.0.

The exact released 0.19.2 native CI passes Linux; both macOS jobs remain running
at final inspection. This uncommitted candidate has no hosted acceptance. Linux local checks
cannot qualify native macOS delivery. Shared Tooling remains exact 0.6.4; remote main
has no newer committed revision at review.

Evidence stays under ignored `target/metrics-070/`. No Git/version/release effect,
production IC operation, sibling edit or evidence cleanup is performed.
Coordination: [Backup #40](https://github.com/dragginzgame/ic-backup/issues/40),
[Metrics #63](https://github.com/dragginzgame/ic-metrics/issues/63).

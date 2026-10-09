# Shared Tooling 0.2.3 adoption — 2026-10-09

Adopt committed `ac4549c5ebde497f7db0da5d05d32835112e51de` from a clean
isolated checkout through the canonical exporter. All 92 selected paths match
committed bytes and executable modes. Four guidance files change:
`docs/consuming-snapshots.md`, `docs/supported-hosts.md`, `tasks/README.md` and
`tasks/ci-health.md`. Engineering rules, executable bytes, tool pins and the
selection roster remain unchanged. No file was patched in place.

Shared #73's optional CI installer fixture declares all five direct wrapper
companions. IC Backup uses the production ShellCheck wrapper/engine and omits
that fixture; no unused wrappers or new local test implementation are introduced.
The exact upstream exporter suite checks refusal of each incomplete selection,
retention of existing consumer bytes and admission/execution of the complete
explicit fixture selection. That owner evidence remains distinct from this
consumer's actual snapshot/pin/link checks. CI-health guidance requires concrete
job/source/input evidence for queue and repeated-validation diagnoses; it does
not authorize cancellations, settings changes or removal of native obligations.

Keep the compatible 0.11.1 draft and all incoming manifest/lock bytes (Host
0.9.2, Testkit 0.27.0, Metrics 0.3.1). Prior original-journal and simulator
qualification remains source/graph-bound; documentation-only adoption requires
no fresh product compilation or simulator launch. The earlier missing selected
Testkit CLI prerequisite is unaffected. No product API, record, outcome, spending,
runner or release authority changes; no function, method or type is removed.

[Qualification](qualification.json) records source/export identities, exact
upstream CI observations and focused owner/consumer checks. Candidate adoption
is local and uncommitted; native upstream acceptance is independent. No full
CI/release gate, root Git write, publication or sibling edit was performed.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Reviewable Changes

## Purpose

A change should deliver one bounded outcome while including all direct proof
and propagation required to make that outcome complete. File count is a signal,
not the unit of work.

This principle combines IcyDB's landing-slice guidance with Canic's distinction
between implementation slices and release batches. Exact turn, continuation,
version, and publication semantics remain consumer policy.

## Coherent outcome

A reviewable change names:

- one bounded outcome and its canonical owner;
- the delivery areas it expects to touch;
- its focused validation boundary;
- relevant public, persisted-format, performance, or artifact-size impact; and
- the directly required tests, diagnostics, documentation, fixtures, and
  mechanical propagation.

Those direct consequences belong with the outcome. Do not create micro-changes
for compiler fallout, exhaustive matches, or a regression test that proves the
same behavior.

Conversely, do not combine independent owners or independently reviewable
outcomes merely to reduce the number of handoffs or validation runs.

## Width and coupling

A wide change can remain coherent when every touched file is required by the
same authority and outcome. A small diff can still be incoherent when it mixes
unrelated behavior.

At handoff, report:

- files touched and approximate line delta;
- primary delivery areas crossed;
- whether width came from semantics or mechanical propagation; and
- whether the implementation became simpler, stayed neutral, or became more
  complex.

Stop and split or update the plan when work reveals another production
behavior, canonical owner, or independently useful result. The cost of another
compile or review is not sufficient reason to hide that outcome in the current
change.

## Validation

Run the narrowest evidence that matches the changed risk while developing.
Broader validation belongs at the consumer's defined readiness or release
boundary. Record both what passed and what was intentionally not run.

Tests protect maintained behavior and trust boundaries, not incidental
implementation shape. Avoid repeating equivalent assertions through every
surface unless the interaction itself can change behavior, recovery, routing,
or cost.

## Consumer-owned policy

Each repository retains authority over:

- how changes map to agent turns, pull requests, commits, or releases;
- the meaning of continuation instructions;
- version and changelog selection;
- broad-suite authorization; and
- closeout, publication, and deployment gates.

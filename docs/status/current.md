# Current handoff — 2026-10-04

The maintainer requested a new repository at `/home/adam/projects/ic-backup`
with instructions and a comprehensive design for extracting backup/restore
from Canic. The target was absent before bootstrap.

This is a documentation-only Git repository with the public remote
[dragginzgame/ic-backup](https://github.com/dragginzgame/ic-backup). The maintainer
authorized its publication and a one-time exception to the no-commit rule for
the initial documentation commit. There are no implementation packages,
executable commands or package releases.
Canic source and its dirty worktree have not been changed by this bootstrap.

Read [the design](../extraction-design.md) for the proposed architecture,
contracts, extraction inventory, recovery cases, testing and implementation
sequence. [The baseline](../source-baseline.json) records source hashes and
the inspected Canic HEAD; its working-tree files, not that commit alone, were
the planning input. These hashes are provenance, not test evidence.

The first implementation batch is B1: refresh the source inventory, trace public
consumers and freeze the current v1 contracts. The current Canic backup crate
has no direct Canic dependencies, but contains Canic-specific Root/Fleet schema
and consistency assumptions. Its live CLI backup preflight remains unavailable.
Neither independence nor live backup availability is established by the design.

The documentation bootstrap is complete. The maintainer subsequently requested
the initial commit and push to the public remote. Extraction implementation,
Canic adoption, package publication and live IC effects need their own
instructions. The standing no-commit rule remains in force for future work.

# Recovery async hard cut and canonical reply evidence

This extends the uncommitted 0.17.0 batch on released 0.16.0,
`943ade5afac218d2eeae280111a75f26612ab80f`. The selected method is
[flow convergence and duplication](../../../../../../../../audits/flow-convergence-and-duplication.md)
at reviewed Shared Tooling 0.3.8, `67285b28a98b7c4211ad32de726709d4e87edea4`,
under the local AGENTS overlay. The scope is the three IC recovery observation ports,
the configured Agent adapter, their consumers and passive reply-evidence framing.
Application fence/membership/consistency acquisition and full runners remain separate
owners; this does not qualify them. The earlier [upload review](../../../10/continuation-0170/01/report.md)
retains proof for its actual prior graph. Package and receipt remain 0.16.0.

| Behavior | Owner and carried contract | Consumers |
| --- | --- | --- |
| Independently reserved recovery | Original request/reservation owner and currently guarded journal; both pending attempt IDs and exact original payload | Three async observation ports and Agent adapter |
| One retained submission | Existing transport preparation/consumed update; mandatory durable signed ingress and never-dispatched custody | Agent recovery status/list/readback |
| Passive response admission | Existing status/list and distinct data-response owners retain 1 MiB/2 MiB bounds | Canonical association/decoders; no receipts |
| Reply evidence | One private hash-framing owner; existing domain, chronological u32 IDs, ingress ID and raw-reply checksum | Mutation, transfer, upload and recovery adapters |

Verdict: **PASS** for the bounded local flow/adoption scope. Native Intel macOS
acceptance and full application custody remain separate, unqualified obligations.

The synchronous recovery interfaces required integrations to bypass the typed ports
or block around async transport. Hard cut all three interfaces to futures and the
supplied current original record. One configured Agent provider converges recovery
submission, while retaining distinct response bounds and separately reserved spending.
No default retainer or application admission exists. Status/controllers, inventory
cardinality and matching bytes remain passive rather than settled mutation outcomes.
The repeated existing hash framing is consolidated with fixed independent byte goldens;
domains and existing evidence identities remain exact. These bounded findings are
resolved in source. The async-boundary finding is MEDIUM: a bounded integration gap
owned by the three core ports and Agent adapter. The duplicated evidence framing is
LOW: limited maintenance friction owned by the private Agent hash helper. Their
dispositions are hard cut and consolidate respectively; frozen v1 records and
independent response/spending owners are retained.

Four actual HTTP cases exercise status, capture list, allocation list and destination
readback across stale-record refusal, retention refusal, accepted/lost/cancelled reply
and retainer-owned repeat-dispatch denial. Exact signed HTTP bodies and original
journal bytes remain. A real simulator trace reserves status after a discarded stop
reply, awaits configured Agent recovery, decodes actual Stopped and leaves both
attempts pending. A second actual trace performs allocation-list recovery and full
1 MiB Wasm data readback through the configured provider, compares exact destination
bytes, preserves both pending attempts and refuses repeat dispatch. Its raw readback
exceeds 1 MiB, exercising the distinct 2 MiB data-response owner. HTTP substitutes
do not qualify management effects. These simulator traces do not qualify generic
application permissions, exclusive allocation attribution, chronology or settlement.

Incoming Host 0.12.10 and Metrics 0.5.6 Rust files match the prior 0.12.8/0.5.5 source.
Testkit remains 0.33.1. Add workspace-inherited dev-only Candid for exact SDK fixture
encoding; existing registry selections remain unchanged. The first HTTP compile
identified that missing direct fixture declaration; initial linting identified doc
markup. Corrected strict Clippy, four HTTP cases, actual reserved-status recovery and
fixed reply-evidence goldens pass. Retain all initial/corrected logs under
`target/continuation-0171/`. `make ci` passes all 16 delivery gates on frozen Host
0.12.10/Metrics 0.5.6 inputs. The sole subsequent technical edit adds the final
simulator trace; the shared validation runner then passes all eight affected goals:
check-doc-links, fmt-check, check, clippy, test, doc, check-msrv and package. Reuse
unchanged shared/tools/pins/deps/shell/tooling/release/hook proof from the full gate.
Final registered membership is 510 core unit cases, 29 core simulator journeys,
15 Agent HTTP cases, six Agent simulator journeys and one Agent unit golden case.
The locked Rust 1.88 normal consumer compiles all three upload coordinators and all
three recovery interfaces with the configured Agent providers on the exact registry
graph subset, without Testkit/PocketIC or Metrics IC features. Manifest/lock,
technical bytes/modes and binary fixtures remain unchanged in the final gate.
[Machine evidence](qualification.json) binds inputs, graph/archives, tools, logs,
registered cases, public consumer and native observations.

Released 0.16.0 [exact-source CI](https://github.com/dragginzgame/ic-backup/actions/runs/38071504142)
passes Linux and Apple Silicon, including native validation and Bash 3.2 tooling.
Intel macOS was cancelled; #32/#33/#36/#37 retain native acceptance. This uncommitted
candidate has no hosted result. #25/#29 retain application-fence/full-runner/terminal
work. New #38 coordinates unreleased Metrics 0.6; its public aggregate/clamped-input
adoption awaits publication. No root Git write, release, registry upload, production
IC, sibling edit or recovery cleanup ran.

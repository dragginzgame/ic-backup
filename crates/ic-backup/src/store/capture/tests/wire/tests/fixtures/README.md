# Independent management wire fixtures

These finite binary inputs replace Backup's direct test-only Candid and management
SDK dependencies (Backup issue #2). They are test data, not a replacement codec.
Ordinary tests replay them without the SDK or runtime schema generation. Normal
policy setup continues to use Protocol's finite response fixture APIs.

Source: Backup `dba14cd3dda9c0d1eff32414b57b86b11f69a97c` (v0.105.1),
Candid 0.10.40 and Management Canister Types 0.8.0. The original test schemas
were independently authored in Backup, not copied from supplier implementations.
`manifest.json` records the original test, zero-based SDK encoding invocation,
length and SHA-256 for each retained vector. The existing literal legacy goldens
remain in their original owning directories with their original hashes.

Original schema owners under `crates/ic-backup/src/store/` at that revision:

- `capture/status/wire/tests.rs`: complete SDK status, authored partial
  `Status`/`Settings` records and explicit `Visibility` variant. State, controllers
  and module hash are authoritative. Resource statistics, version, padding and
  viewer visibility grant no management authority. Null/reserved/absent and
  bare/optional hashes remain distinct. Optional controllers and unknown states
  refuse. The hostile controller/hash serializers declare u32::MAX elements
  without supplying element bodies: refusal must occur at the count limit.
  Padding vectors retain cumulative outer/nested skipped-work boundaries.
- `capture/transfer/metadata/tests.rs`: authored `Shape`, `GlobalsOnly`,
  `RequiredOnly`, `Future`, `BadSource`, empty and zero-labelled-field records.
  Null/reserved/bare options are valid wire but unsupported capture profiles
  refuse. Unknown selected variants and non-reserved source payloads refuse.
  Missing optional fields stay absent; required fields cannot default. V128
  2^128 refuses at the codec limit. A count-only global vector refuses before
  missing elements are read.
- `capture/snapshot/tests.rs`: independently authored `Extra`, `Missing` and
  signed-timestamp `Wrong` records refuse; a valid snapshot plus a second u64
  argument refuses arity. The exact take argument binds the original target,
  no replacement, explicit false uninstall and absent sender version.
- `capture/transfer/readback/tests.rs`: uploaded metadata plus a second u64
  argument, and a u64 instead of metadata, cannot compare as successful readback.
- `restore/procedure/tests.rs`: one u8/null argument is not an empty response.
  Seven readback requests bind the original target, destination snapshot [10],
  selected module/heap/stable extents and ordered nonempty/empty chunks. The
  original SDK decoder assertions checked each target/kind/offset/size/hash;
  during freezing, SDK decode/re-encode also matched every retained byte.
- The upload-metadata request uses the original independent SDK DTO and exact
  byte comparison: target, no replacement, sizes 3/5/7, i32::MIN, i64::MIN,
  F32 NaN bits 0x7fc00042, F64 negative zero bits 0x8000000000000000,
  u128::MAX, certified bytes [17;32], active timer 19 and executed hook.

Inputs were captured by running these owning tests against their existing SDK
projections before removing the dependency. Take arguments were independently
encoded with the SDK and matched; read arguments passed the original independent
field checks and SDK byte comparison. The request comparisons now assert exact
frozen bytes, not a production encoder/decoder round trip. Do not regenerate
expectations automatically from changed product output. Review schema/protocol
changes against the pinned source and independently qualify any replacement.

Local generation and SDK request-proof logs remain under
`target/evidence/sdk-fixture-retirement/`; they are not ordinary test inputs.
No legacy source was copied. Frozen legacy revision
`4ae087fad8c23de712aa60bf569bde4b2b932651`, snapshot reply/metadata wire and
regression owners, supplied the safety review recorded in the transfer contract.

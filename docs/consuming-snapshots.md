# Consuming Shared Tooling Snapshots

CI and release behavior should not depend on a sibling checkout, a moving Git
branch, or network availability. Consumers vendor a reviewed file set and
record its exact Shared Tooling source revision.

## Initial snapshot

Run the refresh helper from a clean Shared Tooling checkout:

```bash
/path/to/shared-tooling/scripts/distribution/refresh-consumer.sh \
  --consumer /path/to/consumer \
  --file scripts/ci/verify-file-checksum.sh \
  --file scripts/ci/verify-shared-tooling-snapshot.sh \
  --file scripts/ci/run-validation-targets.sh
```

The helper validates every source path, copies the files at the same relative
paths, and writes `.shared-tooling.snapshot` in the consumer. The manifest
records format version `1`, source remote, source commit, and the SHA-256 digest
and executable state of every vendored file.

File paths are intentionally identical in source and consumer. A repository
that needs a different path or behavior owns an adapter rather than a patched
shared copy.

## Refresh

After the initial snapshot, omit `--file`. The existing manifest owns the file
set:

```bash
/path/to/shared-tooling/scripts/distribution/refresh-consumer.sh \
  --consumer /path/to/consumer
```

Review the resulting consumer diff normally. Refresh never deletes a file and
does not commit, stage, or push changes.

The source remote must continue to match the manifest exactly. Switching
between SSH, HTTPS, or a fork is an explicit provenance change and requires a
reviewed manifest recreation.

Refresh stages files in a temporary directory under the consumer so the final
manifest replacement stays on the same filesystem. Consumers should ignore
`.shared-tooling-refresh.*` in case an ungraceful process termination prevents
normal cleanup.

To change the declared file set, edit or recreate the manifest as an explicit
reviewed consumer change; ordinary refresh does not silently widen it.

## Drift verification

Consumers that vendor the verifier and checksum helper can check their snapshot
without the Shared Tooling checkout or network access:

```bash
bash scripts/ci/verify-shared-tooling-snapshot.sh
```

The verifier fails when a declared file is absent, symlinked, has different
content, or changes executable state. It validates local snapshot integrity;
the source commit and normal review establish provenance.

## Shared principles

Governance documents may be included in the same manifest. A consumer should
reference the vendored baseline from its local `AGENTS.md` and keep local
architecture, release, deployment, and exception policy outside the vendored
file. Do not edit a vendored shared document in place.

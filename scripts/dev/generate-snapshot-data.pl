#!/usr/bin/env perl
# Independent DIDL, field-hash, LEB128 and SHA-256 vectors for data reads.
use strict;
use warnings;
use JSON::PP qw(decode_json);
use Digest::SHA qw(sha256_hex);
use FindBin;
use lib $FindBin::Bin;
use SnapshotGolden qw(uleb sleb field_hash labels fields variant argument write_json);

die "usage: perl scripts/dev/generate-snapshot-data.pl [--check]\n"
    if @ARGV > 1 || (@ARGV && $ARGV[0] ne '--check');
open my $in, '<', 'crates/ic-backup/src/model/ic_snapshot_metadata/tests/golden.json' or die $!;
my $sources; { local $/; $sources = decode_json(<$in>); }
my ($source) = grep { $_->{name} eq 'data-source' } @$sources;
die 'missing retained metadata fixture' unless $source;
my $target = pack('H*', '00000000000000060101');
my $snapshot = pack('C*', 0,255,17);
my $chunk = $snapshot;
my %args = (canister_id => -24, snapshot_id => 4, kind => 1);
my %kinds = (wasm_module => 2, wasm_memory => 2, stable_memory => 2, wasm_chunk => 3);
my %region = (offset => -8, size => -8);
# SDK encoders retain the first record, then kind variant and its nested records,
# hash vector, and snapshot-ID vector (the latter shares the same vec nat8 type).
my @types = (fields(-20, \%args), fields(-21, \%kinds), fields(-20, \%region),
    fields(-20, {hash => 4}), sleb(-19).sleb(-5));
my @kind_labels = labels(\%kinds);
my $raw = 'DIDL' . uleb(2) . fields(-20, {chunk => 1}) . sleb(-19).sleb(-5)
    . uleb(1) . sleb(0) . uleb(length $chunk) . $chunk;
my $payload_checksum = sha256_hex($raw);
my @cases;
for my $name ('wasm_module', 'wasm_memory', 'stable_memory', 'wasm_chunk') {
    my ($tag) = grep { $kind_labels[$_] eq $name } 0..$#kind_labels;
    my $offset = $name eq 'wasm_module' ? 5 : $name eq 'wasm_memory' ? 17 : 18446744073709551612;
    my $hash = pack('H*', sha256_hex($chunk));
    my $value = $name eq 'wasm_chunk' ? uleb(32) . $hash
        : join('', map { $_ eq 'offset' ? ($name eq 'stable_memory' ? pack('H*', 'fcffffffffffffff') : pack('Q<', $offset)) : pack('Q<', 3) } labels(\%region));
    my %values = (canister_id => "\x01" . uleb(length $target) . $target,
        snapshot_id => uleb(length $snapshot) . $snapshot, kind => uleb($tag) . $value);
    my $arguments = 'DIDL' . uleb(scalar @types) . join('', @types)
        . uleb(1) . sleb(0) . join('', map { $values{$_} } labels(\%args));
    my $method = 'read_canister_snapshot_data';
    my $request_digest = sha256_hex("ic-backup/ic-management-request/v1\0" . "\0"
        . chr(length $target) . $target . "\x01" . chr(length $method) . $method
        . pack('N', length $arguments) . $arguments);
    push @cases, { name => $name, kind => {$name => $name eq 'wasm_chunk' ? {hash => [unpack('C*', $hash)]} : {offset => $offset, size => 3}},
        arguments_hex => unpack('H*', $arguments), request_digest => $request_digest,
        reply_hex => unpack('H*', $raw), payload_checksum => $payload_checksum,
        chunk => [unpack('C*', $chunk)], chunk_checksum => sha256_hex($chunk),
        metadata_digest => $source->{digest},
        digest => sha256_hex("ic-backup/ic-snapshot-data-reply/v1\0" . $source->{digest} . $request_digest . $payload_checksum) };
}
write_json('crates/ic-backup/src/model/ic_snapshot_data/tests/golden.json', \@cases);
write_json('docs/contracts/ic-snapshot-data.json', {
    schema => 1, owner => 'model::ic_snapshot_data',
    kind => 'ephemeral metadata-bound request/reply codec; no persisted record, progress or provider',
    sdk => 'ic-management-canister-types =0.11.0',
    request => { method => 'read_canister_snapshot_data', receiver => 'aaaaa-aa', mode => 'replicated host update ingress',
        metadata => 'borrow exact admitted metadata reply and its canonical target/raw snapshot ID',
        ranges => 'Wasm module, Wasm heap or stable memory: nonzero bounded size, checked offset+size <= retained region size',
        chunks => 'exact 32-byte hash present in retained metadata; no offset or fallback identity',
        hash_owner => 'existing management payload digest; metadata excluded from nonrecursive wire hash' },
    reply => { wire => 'exactly one record with required chunk blob; no skipped fields, extra arguments or trailing bytes',
        ranges => 'actual byte length equals exact requested size', chunks => 'SHA-256 of actual bytes equals exact requested hash; empty known chunks allowed',
        diagnostics => 'typed errors and Debug omit raw bytes, arguments and snapshot/hash identifiers' },
    bounds => { chunk_bytes => 1048576, raw_reply_bytes => 2097152, decoder_work => 8388608,
        skipped_work => 0, type_table_entries => 16, header_bytes => 4096, argument_bytes => 4096, snapshot_id_bytes => 256 },
    hashing => { raw => 'SHA-256 of exact Candid bytes', chunk => 'SHA-256 of actual data bytes',
        domain_utf8_with_nul => "ic-backup/ic-snapshot-data-reply/v1\0",
        body => 'retained metadata evidence hash, exact request hash, raw reply hash; each 64 lowercase ASCII hex bytes' },
    golden_registry => { path => 'crates/ic-backup/src/model/ic_snapshot_data/tests/golden.json',
        generator => 'scripts/dev/generate-snapshot-data.pl',
        metadata_owner => 'scripts/dev/generate-snapshot-metadata.pl data-source fixture',
        construction => 'independent DIDL, field hashes, LEB128, nat64 and SHA-256; every registered case tested' },
    integration_owned => ['authenticated context/target/snapshot association', 'fresh read access, original per-call spending and proof of no prior dispatch',
        'stable complete artifact bytes, durable transfer and authentic capture', 'upload/load safety, custody and terminal/release admission'],
    coverage => { owner => 'model::ic_snapshot_coverage',
        state => 'ephemeral three nat64 region cursors and at most 1024 chunk-presence bits; no data retained',
        association => 'exact original metadata request and raw-reply evidence digest',
        ranges => 'each region starts at zero and advances contiguously; regions may interleave; gaps, overlaps and duplicates reject',
        chunks => 'each metadata hash admits once in any order, including actual empty chunks',
        complete => 'every exact region size and chunk covered; zero-size regions need no read',
        failures => 'coverage unchanged on rejection; reconstruction starts empty, never resumes or resets spending',
        grants => 'no byte custody, durable artifact, authenticated origin, backend transfer, accounting, dispatch or terminal/release' },
    grants => 'no aggregate transfer/progress, complete artifact, fresh permission, spending, receipt, retry/reconciliation, upload/load/start or terminal/release',
    unchanged => 'existing lifecycle/recovery enums/records, metadata owner, request hash, journals, manifests, attempts and source references'
});

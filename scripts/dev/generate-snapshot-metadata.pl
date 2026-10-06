#!/usr/bin/env perl
# Independent DIDL/field-hash/LEB128 goldens. No Rust encoder or IC calls.
use strict;
use warnings;
use JSON::PP;
use Digest::SHA qw(sha256_hex);
use Math::BigInt;

sub uleb {
    my $n = Math::BigInt->new("$_[0]");
    my $bytes = '';
    do {
        my $b = $n->copy()->band(127)->numify();
        $n->brsft(7);
        $bytes .= chr($b | ($n->is_zero() ? 0 : 128));
    } until ($n->is_zero());
    return $bytes;
}
sub sleb {
    my $n = shift;
    my $bytes = '';
    while (1) {
        my $b = $n & 127;
        $n = int(($n - $b) / 128);
        my $done = ($n == 0 && !($b & 64)) || ($n == -1 && ($b & 64));
        $bytes .= chr($b | ($done ? 0 : 128));
        last if $done;
    }
    return $bytes;
}
sub field_hash {
    my $hash = 0;
    $hash = ($hash * 223 + $_) % 4294967296 for unpack('C*', $_[0]);
    return $hash;
}
sub labels { return sort { field_hash($a) <=> field_hash($b) } keys %{$_[0]}; }
sub fields {
    my ($tag, $fields) = @_;
    return sleb($tag) . uleb(scalar keys %$fields) . join('',
        map { uleb(field_hash($_)) . sleb($fields->{$_}) } labels($fields));
}
sub variant {
    my ($fields, $name, $value) = @_;
    my @labels = labels($fields);
    my ($index) = grep { $labels[$_] eq $name } 0..$#labels;
    return uleb($index) . $value;
}
sub write_json {
    my ($path, $value) = @_;
    my $bytes = JSON::PP->new->canonical->pretty->encode($value);
    if (@ARGV && $ARGV[0] eq '--check') {
        open my $in, '<', $path or die "$path: $!";
        local $/;
        die "generated content differs: $path\n" if <$in> ne $bytes;
    } else {
        open my $out, '>', $path or die "$path: $!";
        print {$out} $bytes;
    }
}
die "usage: perl scripts/dev/generate-snapshot-metadata.pl [--check]\n"
    if @ARGV > 1 || (@ARGV && $ARGV[0] ne '--check');

my $target_bytes = pack('H*', '00000000000000060101');
my $snapshot_bytes = pack('C*', 0, 255, 17);
my %argument_fields = (canister_id => -24, snapshot_id => 1);
my %argument_values = (canister_id => "\x01" . uleb(length $target_bytes) . $target_bytes,
    snapshot_id => uleb(length $snapshot_bytes) . $snapshot_bytes);
my $arguments = 'DIDL' . uleb(2) . fields(-20, \%argument_fields) . sleb(-19) . sleb(-5)
    . uleb(1) . sleb(0) . join('', map { $argument_values{$_} } labels(\%argument_fields));
my $method = 'read_canister_snapshot_metadata';
my $request_digest = sha256_hex("ic-backup/ic-management-request/v1\0" . "\0"
    . chr(length $target_bytes) . $target_bytes . "\x01" . chr(length $method)
    . $method . pack('N', length $arguments) . $arguments);
my %source = (taken_from_canister => -16, metadata_upload => -16);
my %global = (i32 => -11, i64 => -12, f32 => -13, f64 => -14, v128 => -3);
my %timer = (inactive => -1, active => -8);
my %hook = (condition_not_satisfied => -1, ready => -1, executed => -1);
my %metadata = (source => 2, taken_at_timestamp => -8, wasm_module_size => -8,
    globals => 5, wasm_memory_size => -8, stable_memory_size => -8,
    wasm_chunk_store => 7, canister_version => -8, certified_data => 0,
    global_timer => 9, on_low_wasm_memory_hook_status => 11);
my @common_types = (sleb(-19).sleb(-5), fields(-21, \%source), sleb(-18).sleb(1),
    fields(-21, \%global), sleb(-18).sleb(3), sleb(-19).sleb(4),
    fields(-20, {hash => 0}), sleb(-19).sleb(6), fields(-21, \%timer),
    sleb(-18).sleb(8), fields(-21, \%hook), sleb(-18).sleb(10));
my $max64 = "\xff" x 8;
my @global_values = (variant(\%global, 'i32', pack('H*', '00000080')),
    variant(\%global, 'i64', pack('H*', '0000000000000080')),
    variant(\%global, 'f32', pack('H*', '4200c07f')),
    variant(\%global, 'f64', pack('H*', '0000000000000080')),
    variant(\%global, 'v128', uleb('340282366920938463463374607431768211455')));
my %values = (source => "\x01" . variant(\%source, 'taken_from_canister', ''),
    taken_at_timestamp => $max64, wasm_module_size => $max64,
    globals => uleb(6) . join('', map { "\x01" . $_ } @global_values) . "\0",
    wasm_memory_size => "\0" x 8, stable_memory_size => $max64,
    wasm_chunk_store => uleb(2) . uleb(32) . ("\x02" x 32) . uleb(32) . ("\x01" x 32),
    canister_version => $max64, certified_data => uleb(32) . ("\x11" x 32),
    global_timer => "\x01" . variant(\%timer, 'active', $max64),
    on_low_wasm_memory_hook_status => "\x01" . variant(\%hook, 'executed', ''));
my @cases;
for my $name ('sdk-optional', 'required-source-globals', 'unavailable', 'data-source') {
    my @types = @common_types;
    my %fields = %metadata;
    my %data = %values;
    if ($name eq 'required-source-globals') {
        $fields{source} = 1;
        $types[5] = sleb(-19).sleb(3);
        $data{source} = variant(\%source, 'taken_from_canister', '');
        $data{globals} = uleb(5) . join('', @global_values);
    } elsif ($name eq 'unavailable') {
        $data{source} = "\0"; $data{globals} = uleb(1) . "\0";
        $data{global_timer} = "\0"; $data{on_low_wasm_memory_hook_status} = "\0";
        $data{wasm_chunk_store} = uleb(0); $data{certified_data} = uleb(0);
    } elsif ($name eq 'data-source') {
        $data{wasm_module_size} = pack('Q<', 64);
        $data{wasm_memory_size} = pack('Q<', 64);
        $data{wasm_chunk_store} = uleb(2)
            . uleb(32) . pack('H*', sha256_hex($snapshot_bytes))
            . uleb(32) . pack('H*', sha256_hex(''));
    }
    push @types, fields(-20, \%fields);
    my $raw = 'DIDL' . uleb(scalar @types) . join('', @types) . uleb(1) . sleb(12)
        . join('', map { $data{$_} } labels(\%fields));
    my $checksum = sha256_hex($raw);
    push @cases, { name => $name, target => 'renrk-eyaaa-aaaaa-aaada-cai',
        snapshot_id => [0,255,17], arguments_hex => unpack('H*', $arguments),
        request_digest => $request_digest, reply_hex => unpack('H*', $raw),
        payload_checksum => $checksum,
        digest => sha256_hex("ic-backup/ic-snapshot-metadata-reply/v1\0" . $request_digest . $checksum),
        globals => $name eq 'unavailable' ? 1 : $name eq 'required-source-globals' ? 5 : 6,
        source_present => $name eq 'unavailable' ? JSON::PP::false : JSON::PP::true };
}
write_json('crates/ic-backup/src/model/ic_snapshot_metadata/tests/golden.json', \@cases);
write_json('docs/contracts/ic-snapshot-metadata.json', {
    schema => 1, owner => 'model::ic_snapshot_metadata',
    kind => 'ephemeral metadata request/reply codec; no persisted product record or provider',
    sdk => 'ic-management-canister-types =0.11.0',
    request => { method => $method, receiver => 'aaaaa-aa', mode => 'replicated host update ingress',
        arguments => 'exact canonical canister_id and 1..256 raw snapshot_id bytes',
        hash_owner => 'existing ic-management-request/v1 digest; six-method v1 record unchanged' },
    fields => { required => [qw(taken_at_timestamp wasm_module_size globals wasm_memory_size stable_memory_size wasm_chunk_store canister_version certified_data)],
        optional => [qw(source global_timer on_low_wasm_memory_hook_status)],
        globals => 'ordered optional SDK globals; preserve absent slots and f32/f64 bits, reject v128 above 128 bits',
        chunks => 'ordered distinct exact 32-byte SHA-256 identities',
        integers => 'required nat64 fields retain full range; no size sum or total-transfer inference',
        absence => 'absent optional values remain absent, not defaults, upload inputs or qualification' },
    bounds => { raw_bytes => 1048576, globals => 4096, chunks => 1024, certified_data_bytes => 32,
        snapshot_id_bytes => 256, argument_bytes => 4096, decoder_work => 2097152,
        skipped_work => 0, type_table_entries => 64, header_bytes => 16384 },
    decoding => 'one argument, bounded visitors without untrusted preallocation, no skipped values/extensions/unknown optional coercion or trailing bytes; actual reserved source marker adapter',
    evidence => { raw => 'SHA-256 of exact raw bytes including order and absence',
        domain_utf8_with_nul => "ic-backup/ic-snapshot-metadata-reply/v1\0",
        body => '64 lowercase ASCII request hash then 64 lowercase ASCII raw reply hash' },
    golden_registry => { path => 'crates/ic-backup/src/model/ic_snapshot_metadata/tests/golden.json',
        generator => 'scripts/dev/generate-snapshot-metadata.pl',
        construction => 'independent DIDL, field hashes, LEB128, numeric/float bytes and SHA-256; all registered cases tested' },
    grants => 'no fresh read permission, signing, dispatch, spending, complete transfer/upload, receipt, reconciliation, terminal or release',
    integration_owned => ['authenticated network/caller/target/snapshot association', 'fresh snapshot-read access and original per-call accounting',
        'data extents, authentic capture and stable bytes', 'upload/load/start safety and application/command custody']
});

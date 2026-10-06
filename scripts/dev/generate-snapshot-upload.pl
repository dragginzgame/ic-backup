#!/usr/bin/env perl
# Independent DIDL/field-hash/LEB128/float-bit/SHA-256 upload vectors. No Rust encoder.
use strict;
use warnings;
use JSON::PP;
use Digest::SHA qw(sha256_hex);
use Math::BigInt;

sub uleb {
    my $n = Math::BigInt->new("$_[0]"); my $bytes = '';
    do { my $b = $n->copy()->band(127)->numify(); $n->brsft(7);
        $bytes .= chr($b | ($n->is_zero() ? 0 : 128)); } until ($n->is_zero());
    return $bytes;
}
sub sleb {
    my $n = shift; my $bytes = '';
    while (1) { my $b = $n & 127; $n = int(($n - $b) / 128);
        my $done = ($n == 0 && !($b & 64)) || ($n == -1 && ($b & 64));
        $bytes .= chr($b | ($done ? 0 : 128)); last if $done; }
    return $bytes;
}
sub field_hash { my $h = 0; $h = ($h * 223 + $_) % 4294967296 for unpack('C*', $_[0]); return $h; }
sub labels { return sort { field_hash($a) <=> field_hash($b) } keys %{$_[0]}; }
sub fields {
    my ($tag, $fields) = @_;
    return sleb($tag) . uleb(scalar keys %$fields) . join('',
        map { uleb(field_hash($_)) . sleb($fields->{$_}) } labels($fields));
}
sub variant {
    my ($fields, $name, $value) = @_; my @labels = labels($fields);
    my ($index) = grep { $labels[$_] eq $name } 0..$#labels;
    return uleb($index) . $value;
}
sub argument {
    my ($types, $fields, $values) = @_;
    return 'DIDL' . uleb(scalar @$types) . join('', @$types) . uleb(1) . sleb(0)
        . join('', map { $values->{$_} } labels($fields));
}
sub write_json {
    my ($path, $value) = @_; my $bytes = JSON::PP->new->canonical->pretty->encode($value);
    if (@ARGV && $ARGV[0] eq '--check') {
        open my $in, '<', $path or die "$path: $!"; local $/;
        die "generated content differs: $path\n" if <$in> ne $bytes;
    } else { open my $out, '>', $path or die "$path: $!"; print {$out} $bytes; }
}
die "usage: perl scripts/dev/generate-snapshot-upload.pl [--check]\n"
    if @ARGV > 1 || (@ARGV && $ARGV[0] ne '--check');
open my $in, '<', 'crates/ic-backup/src/model/ic_snapshot_metadata/tests/golden.json' or die $!;
my $sources; { local $/; $sources = decode_json(<$in>); }
my $target = pack('H*', '00000000000000060101');
my $destination = pack('C*',21,0,255);
my $chunk = pack('C*',0,255,17);
my %global = (i32 => -11, i64 => -12, f32 => -13, f64 => -14, v128 => -3);
my %timer = (inactive => -1, active => -8);
my %hook = (condition_not_satisfied => -1, ready => -1, executed => -1);
my %metadata = (canister_id=>-24, replace_snapshot=>3, globals=>1, certified_data=>4,
    global_timer=>5, on_low_wasm_memory_hook_status=>7,
    wasm_module_size=>-8, wasm_memory_size=>-8, stable_memory_size=>-8);
# Canonical SDK DFS type registration, with shared vec nat8 and exact variant labels.
my @metadata_types = (fields(-20,\%metadata), sleb(-19).sleb(2), fields(-21,\%global),
    sleb(-18).sleb(4), sleb(-19).sleb(-5), sleb(-18).sleb(6), fields(-21,\%timer),
    sleb(-18).sleb(8), fields(-21,\%hook));
my @globals = (variant(\%global,'i32',pack('H*','00000080')),
    variant(\%global,'i64',pack('H*','0000000000000080')),
    variant(\%global,'f32',pack('H*','4200c07f')),
    variant(\%global,'f64',pack('H*','0000000000000080')),
    variant(\%global,'v128',uleb('340282366920938463463374607431768211455')));
my @cases;
sub register {
    my ($name,$source,$method,$args,$raw,$extra)=@_;
    my ($metadata_source)=grep { $_->{name} eq $source } @$sources;
    die "missing source: $source" unless $metadata_source;
    push @cases, { name=>$name, source=>$source, metadata_digest=>$metadata_source->{digest},
        method=>$method, arguments_hex=>unpack('H*',$args),
        request_digest=>sha256_hex("ic-backup/ic-management-request/v1\0" . "\0" . chr(length $target)
            . $target . "\x01" . chr(length $method) . $method . pack('N',length $args) . $args),
        reply_hex=>unpack('H*',$raw), payload_checksum=>sha256_hex($raw), %$extra };
}
my $metadata_reply = 'DIDL'.uleb(2).fields(-20,{snapshot_id=>1}).sleb(-19).sleb(-5)
    .uleb(1).sleb(0).uleb(length $destination).$destination;
for my $source ('upload-source','upload-source-absent') {
    my $absent = $source eq 'upload-source-absent';
    my %values=(canister_id=>"\x01".uleb(length $target).$target,replace_snapshot=>"\0",
        globals=>uleb(5).join('',@globals),certified_data=>uleb(32).("\x11"x32),
        wasm_module_size=>pack('Q<',64),wasm_memory_size=>pack('Q<',64),stable_memory_size=>"\xff"x8,
        global_timer=>$absent ? "\0" : "\x01".variant(\%timer,'active',"\xff"x8),
        on_low_wasm_memory_hook_status=>$absent ? "\0" : "\x01".variant(\%hook,'executed',''));
    register($source,$source,'upload_canister_snapshot_metadata',argument(\@metadata_types,\%metadata,\%values),$metadata_reply,{});
}
my %data=(canister_id=>-24,snapshot_id=>1,kind=>2,chunk=>1);
my %kinds=(wasm_module=>3,wasm_memory=>3,stable_memory=>3,wasm_chunk=>-1);
my @data_types=(fields(-20,\%data),sleb(-19).sleb(-5),fields(-21,\%kinds),fields(-20,{offset=>-8}));
for my $name ('wasm_module','wasm_memory','stable_memory','wasm_chunk','empty_chunk') {
    my $empty=$name eq 'empty_chunk'; my $kind=$empty ? 'wasm_chunk' : $name;
    my $bytes=$empty ? '' : $chunk;
    my $offset=$kind eq 'wasm_module' ? 5 : $kind eq 'wasm_memory' ? 17 : 18446744073709551612;
    my $value=$kind eq 'wasm_chunk' ? '' : $kind eq 'stable_memory' ? pack('H*','fcffffffffffffff') : pack('Q<',$offset);
    my %values=(canister_id=>"\x01".uleb(length $target).$target,snapshot_id=>uleb(length $destination).$destination,
        chunk=>uleb(length $bytes).$bytes,kind=>variant(\%kinds,$kind,$value));
    register($name,'upload-source','upload_canister_snapshot_data',argument(\@data_types,\%data,\%values),'DIDL'."\0\0",{
        source_kind=>{$kind=>$kind eq 'wasm_chunk' ? {hash=>[unpack('C*',pack('H*',sha256_hex($bytes)))]} : {offset=>$offset,size=>3}},
        chunk=>[unpack('C*',$bytes)],chunk_checksum=>sha256_hex($bytes),destination_id=>[unpack('C*',$destination)]});
}
# Fixed original-plan fixture uses its documented canonical inventory/graph/plan
# encodings independently. These bytes are fixtures, never production encoders.
my $target_text='renrk-eyaaa-aaaaa-aaada-cai';
my $caller='2vxsx-fae';
my $inventory_digest=sha256_hex("ic-backup/inventory/v1\0" . pack('N',1)
    .pack('N',length $target_text).$target_text."\0\0\0");
my $graph_digest=sha256_hex("ic-backup/effect-graph/v1\0" . pack('NQ>N',1,7,0));
my $source_plan_digest=sha256_hex("ic-backup/operation-plan/v1\0" . ('ab'x32)
    .chr(length $caller).$caller.('cd'x32).$inventory_digest.$graph_digest
    .pack('N',1).chr(length $target_text).$target_text.pack('NNNQ>',1,1,1,7)
    .chr(length $target_text).$target_text.('ef'x32).pack('NN',1,1));
my $source_checksum=sha256_hex('independent declared source tree');
for my $case (@cases) {
    my $data=exists $case->{source_kind};
    my ($metadata)=grep {$_->{name} eq $case->{source}} @cases;
    my $tag=$data ? "\x01".$metadata->{request_digest} : "\0";
    $case->{source_plan_digest}=$source_plan_digest;
    $case->{source_checksum}=$source_checksum;
    $case->{binding_digest}=sha256_hex("ic-backup/ic-snapshot-upload-binding/v1\0"
        .$source_plan_digest.$case->{metadata_digest}.$source_checksum.$tag.$case->{request_digest});
    $case->{digest}=sha256_hex("ic-backup/ic-snapshot-upload-reply/v1\0"
        .$case->{binding_digest}.$case->{payload_checksum});
}
write_json('crates/ic-backup/src/model/ic_snapshot_upload/tests/golden.json',\@cases);
write_json('docs/contracts/ic-snapshot-upload.json',{
    schema=>1,owner=>'model::ic_snapshot_upload; policy::ic_snapshot_upload; DownloadJournalGuard preparation',
    sdk=>'ic-management-canister-types =0.11.0',kind=>'ephemeral codecs and passive reservation/reply association; no persisted schema, provider or new progress/accounting owner',
    source=>'full original retained source plan, exact metadata reply and durable artifact checksum; same canonical canister and original network/release',
    metadata=>'preserve ordered globals and floating bits; unavailable global rejects; optional timer/hook remains absent; replace_snapshot always None',
    data=>'exact new 1..256-byte destination distinct from source; checked original region slice or known SHA-256 chunk including empty; upload chunk wire has no source hash',
    preparation=>'explicit fresh full selected-set original plan/journal admission and IC-tree verification; data verifies tree before/after no-follow descriptor read; no future noncooperating byte custody',
    accounting=>'retain binding_digest in an independently retained original upload plan before reserving its mutation; each later data payload includes actual allocated ID before its own intent/spending; metadata allowance never supplies data allowance',
    bounds=>{chunk_bytes=>1048576,argument_bytes=>2097152,raw_reply_bytes=>4096,snapshot_id_bytes=>256,decoder_work=>65536,skipped_work=>0,type_table_entries=>16,header_bytes=>4096},
    hashing=>{wire=>'existing ic-management-request/v1 owner, before upload plan derivation',binding_domain_with_nul=>"ic-backup/ic-snapshot-upload-binding/v1\0",
        binding_body=>'64 ASCII lowercase hashes: full source plan, metadata evidence, artifact checksum; then byte 0 metadata or byte 1 plus metadata wire hash for data; then current wire hash',
        reply_domain_with_nul=>"ic-backup/ic-snapshot-upload-reply/v1\0",reply_body=>'64 ASCII binding hash followed by 64 ASCII exact raw-reply SHA-256'},
    reply=>'bounded exact metadata raw ID or canonical empty data tuple; current original authority/pending mutation and actual claimed context/target must match; no receipt or new destination authority',
    loss=>'lost reply remains pending; provider failure, absent data or inventory cardinality supplies no NotApplied/Uncertain result; separately qualified settled reconciliation required before any new authority',
    golden_registry=>{path=>'crates/ic-backup/src/model/ic_snapshot_upload/tests/golden.json',generator=>'scripts/dev/generate-snapshot-upload.pl',construction=>'independent DIDL, field hashes, LEB128, exact global bits and SHA-256; every registered case tested'},
    integration_owned=>['authenticated authentic complete source and new snapshot attribution','proof of no prior dispatch and original per-call spending','fresh controllers/permissions, byte and command custody','upload completeness and lost-effect settlement','same-release application/fence/load/start safety and terminal/reference release'],
    grants=>'no replacement/deletion, remote IO, provider, dispatch, retry, refund, journal transition, complete upload, load/start, fence or reference release',
    unchanged=>'existing v1 records, exhaustive lifecycle/recovery enums, journals, allowances, artifacts, wire codecs, package version and release receipt'
});

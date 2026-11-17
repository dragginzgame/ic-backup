#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
export PATH="$root/.tools/host/bin:$PATH"
mkdir -p target/package-consumer
evidence="$(mktemp -d "$root/target/package-consumer/check.XXXXXX")"
printf 'Package consumer evidence: %s\n' "$evidence"
cargo metadata --locked --offline --no-deps --format-version 1 > "$evidence/metadata.json"
version="$(jq -er '.packages[] | select(.name == "ic-backup") | .version' "$evidence/metadata.json")"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
# Working-source qualification is useful before a release commit. Upload's
# independent clean-source check is never relaxed by this package command.
cargo package --locked --offline --allow-dirty -p ic-backup --target-dir "$evidence/archive" > "$evidence/package.log" 2>&1
package="$evidence/archive/package/ic-backup-$version"
cmp crates/ic-backup/README.md "$package/README.md"
test -s "$package/LICENSE.md"
yq -p toml -o json '.' qualification/Cargo.toml > "$evidence/workspace.json"
yq -p toml -o json '.' qualification/ic-backup-consumer/Cargo.toml > "$evidence/consumer.json"
perl -MJSON::PP - "$evidence/workspace.json" "$evidence/consumer.json" \
    "$package" "$version" "$root/qualification/ic-backup-consumer/src/main.rs" > "$evidence/Cargo.toml" <<'PERL'
use strict;
use warnings;
sub read_json {
    open my $f, '<', shift or die $!;
    local $/;
    return decode_json(<$f>);
}
sub value {
    my ($v) = @_;
    return '{ '.join(', ', map { encode_json($_).' = '.value($v->{$_}) } sort keys %$v).' }'
        if ref($v) eq 'HASH';
    return encode_json($v);
}
my ($workspace, $consumer, $path, $version, $source) = @ARGV;
my $catalog = read_json($workspace)->{workspace}{dependencies};
my $dependencies = read_json($consumer)->{dependencies};
$catalog->{'ic-backup'} = {path => $path, version => "=$version"};
print "[workspace]\n[package]\nname = \"backup-package-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n";
print "[[bin]]\nname = \"backup-package-consumer\"\npath = ", value($source), "\n[profile.dev]\ndebug = 0\n[dependencies]\n";
for my $name (sort keys %$dependencies) {
    exists $catalog->{$name} or die "missing dependency $name\n";
    print value($name), ' = ', value($catalog->{$name}), "\n";
}
PERL
cp qualification/Cargo.lock "$evidence/Cargo.lock"
# Only the disposable consumer lock is projected; both maintained locks stay intact.
cargo metadata --offline --manifest-path "$evidence/Cargo.toml" --format-version 1 > "$evidence/resolved.json"
cargo tree --locked --offline --manifest-path "$evidence/Cargo.toml" --edges normal > "$evidence/graph.txt"
if grep -E 'ic-testkit|pocket-ic|ic-backup-qualification' "$evidence/graph.txt"; then
    echo 'qualification dependency leaked into archive consumer' >&2
    exit 1
fi
cargo build --locked --offline --manifest-path "$evidence/Cargo.toml" \
    --target-dir "$root/target/package-consumer/build" > "$evidence/build.log" 2>&1
printf 'Archive and normal-dependency consumer verified: ic-backup %s\n' "$version"

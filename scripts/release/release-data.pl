#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP;
use Digest::SHA qw(sha256_hex);

sub read_file {
    my ($path) = @_;
    open my $fh, '<', $path or die "$path: $!\n";
    local $/;
    return <$fh>;
}

sub write_file {
    my ($path, $text) = @_;
    open my $fh, '>', $path or die "$path: $!\n";
    print {$fh} $text;
    close $fh or die "$path: $!\n";
}

sub version {
    my $text = read_file('Cargo.toml');
    my $package = read_file('crates/ic-backup/Cargo.toml');
    $package =~ /^\[package\]\n(.*?)(?=^\[|\z)/ms
        or die "missing package\n";
    $1 =~ /^version\.workspace = true$/m
        or die "package version must inherit from workspace\n";
    $text =~ /^\[workspace\.package\]\n(.*?)(?=^\[|\z)/ms
        or die "missing workspace package\n";
    my $section = $1;
    $section =~ /^version = "([^"]+)"$/m or die "missing workspace package version\n";
    return $1;
}

sub parts {
    my ($value) = @_;
    $value =~ /\A(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\z/
        or die "expected canonical x.y.z with components below 1 billion\n";
    return ($1, $2, $3);
}

sub next_version {
    my ($requested) = @_;
    my @old = parts(version());
    my @new;
    if ($requested eq 'patch') { @new = ($old[0], $old[1], $old[2] + 1); }
    elsif ($requested eq 'minor') { @new = ($old[0], $old[1] + 1, 0); }
    elsif ($requested eq 'major') { @new = ($old[0] + 1, 0, 0); }
    else { @new = parts($requested); }
    my $next = join '.', @new;
    parts($next);
    for my $i (0..2) {
        last if $new[$i] > $old[$i];
        die "target version must not decrease\n" if $new[$i] < $old[$i];
    }
    return $next;
}

sub compare_versions {
    my ($left, $right) = @_;
    my @left = parts($left);
    my @right = parts($right);
    for my $i (0..2) {
        my $order = $left[$i] <=> $right[$i];
        return $order if $order;
    }
    return 0;
}

sub changelog {
    my ($target, $date) = @_;
    parts($target);
    $date =~ /\A[0-9]{4}-[0-9]{2}-[0-9]{2}\z/ or die "invalid release date\n";
    my $text = read_file('CHANGELOG.md');
    my @sections = $text =~ /^## \[([^\]]+)\](?: - [0-9]{4}-[0-9]{2}-[0-9]{2})?$/mg;
    my %seen;
    for my $section (@sections) {
        die "duplicate changelog section: $section\n" if $seen{$section}++;
        parts($section) unless $section eq 'Draft';
    }
    die "changelog must have a current draft\n" unless @sections;
    my $draft = $sections[0];
    # A release command owns the final version. A numbered future draft is a
    # provisional label; never reinterpret imported history as new release notes.
    die "top section is historical; add an undated draft above it\n"
        if $draft ne 'Draft' && $draft ne $target
            && compare_versions($draft, version()) <= 0;
    die "another draft is open\n" if $draft ne 'Draft' && $seen{Draft};
    die "selected version already has a section\n" if $draft ne $target && $seen{$target};
    $text =~ /^## \[\Q$draft\E\]\n(.*?)(?=^## \[|\z)/ms
        or die "release draft must be undated\n";
    my $notes = $1;
    die "release is already dated\n"
        if $text =~ /^## \[\Q$target\E\] - /m;
    # Imported history can be undated. Only versions newer than the current
    # package are competing drafts; never rewrite historical release notes.
    my @drafts = $text =~ /^## \[([0-9.]+)\]$/mg;
    die "another numbered release draft is open\n"
        if grep { $_ ne $draft && compare_versions($_, version()) > 0 } @drafts;
    die "release draft notes are empty\n" unless $notes =~ /\S/;
    $text =~ s/^## \[\Q$draft\E\]$/## [$target] - $date/m;
    return $text;
}

my ($command, @args) = @ARGV;
$command //= '';
if ($command eq 'version') {
    my $value = version();
    parts($value);
    print "$value\n";
} elsif ($command eq 'next') {
    print next_version($args[0] // ''), "\n";
} elsif ($command eq 'changelog-check' || $command eq 'finalize') {
    my $text = changelog(@args);
    write_file('CHANGELOG.md', $text) if $command eq 'finalize';
} elsif ($command eq 'set-version') {
    my ($target) = @args;
    parts($target);
    my $text = read_file('Cargo.toml');
    version();
    $text =~ s/(^\[workspace\.package\]\n(?:(?!^\[).)*?^version = ")[^"]+(")$/$1$target$2/ms
        or die "cannot update workspace package version\n";
    write_file('Cargo.toml', $text);
} elsif ($command eq 'receipt') {
    my ($source, $date) = @args;
    my %hashes = map { $_ => sha256_hex(read_file($_)) }
        qw(Cargo.toml Cargo.lock crates/ic-backup/Cargo.toml CHANGELOG.md);
    my $receipt = {
        schema => 1, version => version(), source => $source, date => $date,
        gate => 'release-verify', files => \%hashes,
    };
    write_file('docs/release.json', JSON::PP->new->canonical->pretty->encode($receipt));
} elsif ($command eq 'verify' || $command eq 'source') {
    my $receipt = decode_json(read_file('docs/release.json'));
    die "unsupported release receipt\n" unless $receipt->{schema} == 1;
    die "release version mismatch\n" unless $receipt->{version} eq version();
    die "invalid release source\n" unless $receipt->{source} =~ /\A[0-9a-f]{40,64}\z/;
    die "invalid release gate\n" unless $receipt->{gate} eq 'release-verify';
    die "invalid release date\n" unless $receipt->{date} =~ /\A[0-9]{4}-[0-9]{2}-[0-9]{2}\z/;
    my @files = sort keys %{$receipt->{files}};
    die "unexpected receipt files\n"
        unless join(',', @files) eq 'CHANGELOG.md,Cargo.lock,Cargo.toml,crates/ic-backup/Cargo.toml';
    for my $path (@files) {
        die "release file changed after validation: $path\n"
            unless sha256_hex(read_file($path)) eq $receipt->{files}{$path};
    }
    my $notes = read_file('CHANGELOG.md');
    my $heading = "## [$receipt->{version}] - $receipt->{date}";
    die "dated changelog missing\n" unless $notes =~ /^\Q$heading\E$/m;
    print "$receipt->{source}\n" if $command eq 'source';
} else {
    die "unknown release-data command\n";
}

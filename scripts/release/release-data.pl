#!/usr/bin/env perl
# Shared companions: scripts/ci/StableVersion.pm
use strict;
use warnings;
use JSON::PP;
use Digest::SHA qw(sha256_hex);
use File::Path qw(make_path remove_tree);
use File::Temp qw(tempdir);
use FindBin;
use lib "$FindBin::Bin/../ci";
use StableVersion qw(version_cmp);
use Cwd qw(abs_path);
use Encode qw(decode FB_CROAK LEAVE_SRC);

my $read_commit;

sub read_file {
    my ($path) = @_;
    if (defined $read_commit && $path =~ /\A(?:Cargo\.toml|Cargo\.lock|CHANGELOG\.md|docs\/release\.json|crates\/[A-Za-z0-9_-]+\/(?:Cargo\.toml|src\/lib\.rs))\z/) {
        open my $object, '-|', 'git', 'show', "$read_commit:$path"
            or die "cannot read selected release file: $path\n";
        my $text = do { local $/; <$object> };
        close $object or die "selected release file unavailable: $path\n";
        return $text;
    }
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

# The same reviewed TOML projector selects members and names; Cargo owns validity.
sub read_manifest {
    my ($path) = @_;
    my $temporary = tempdir('ic-backup-members.XXXXXX', TMPDIR => 1, CLEANUP => 0);
    write_file("$temporary/input.toml", read_file($path));
    open my $parser, '-|', ($ENV{YQ} // 'yq'), '-p', 'toml', '-o', 'json', '.', "$temporary/input.toml"
        or die "cannot project selected manifest; retained: $temporary\n";
    my $json = do { local $/; <$parser> };
    close $parser or die "selected manifest projection failed; retained: $temporary\n";
    my $value = JSON::PP->new->allow_nonref->decode($json);
    remove_tree($temporary);
    return $value;
}
sub member_manifests {
    my ($manifest) = @_;
    $manifest //= read_manifest('Cargo.toml');
    my $members = $manifest->{workspace}{members};
    ref($members) eq 'ARRAY' && @$members or die "expected explicit workspace members\n";
    my %seen;
    for (@$members) {
        /\Acrates\/[A-Za-z0-9_-]+\z/ && !$seen{$_}++ or die "invalid library workspace member\n";
    }
    return map { "$_/Cargo.toml" } @$members;
}
sub receipt_files { return sort ('Cargo.toml', 'Cargo.lock', 'CHANGELOG.md', member_manifests()); }

sub version {
    my $manifest = 'Cargo.toml';
    my ($export, $value);
    eval {
        if (defined $read_commit) {
            # The shared reader owns TOML parsing, not Git selection. Export the
            # exact selected manifests with their real library target for Cargo's
            # offline structural validation; never borrow working metadata.
            $export = tempdir('ic-backup-version.XXXXXX', TMPDIR => 1, CLEANUP => 0);
            write_file("$export/Cargo.toml", read_file('Cargo.toml'));
            for my $member (member_manifests()) {
                (my $directory = $member) =~ s{/Cargo\.toml$}{};
                make_path("$export/$directory/src");
                write_file("$export/$member", read_file($member));
                write_file("$export/$directory/src/lib.rs", read_file("$directory/src/lib.rs"));
            }
            $manifest = "$export/Cargo.toml";
        }
        open my $reader, '-|', 'bash', "$FindBin::Bin/../ci/read-cargo-workspace-version.sh", '--stable', $manifest
            or die "cannot invoke shared workspace version reader: $!\n";
        $value = do { local $/; <$reader> };
        close $reader or die "shared workspace version reader failed\n";
        die "shared workspace version reader returned no version\n" unless defined $value;
        chomp $value;
        parts($value); # Consumer release arithmetic retains its component ceiling.
        1;
    } or do {
        my $error = $@;
        die $error . (defined $export ? "Selected version export retained: $export\n" : '');
    };
    remove_tree($export) if defined $export;
    return $value;
}

sub parts {
    my ($value) = @_;
    $value =~ /\A(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\z/
        or die "expected canonical x.y.z with components below 1 billion\n";
    return ($1, $2, $3);
}

sub next_version {
    my ($requested) = @_;
    my $previous = version();
    parts($previous);
    my $next = $requested;
    if ($requested =~ /\A(?:patch|minor|major)\z/) {
        open my $helper, '-|', 'bash', 'scripts/ci/next-release-version.sh', $previous, $requested
            or die "cannot inspect common release increment\n";
        $next = do { local $/; <$helper> };
        close $helper or die "common release increment rejected\n";
        chomp $next;
    }
    parts($next);
    die "target version must not decrease\n" if version_cmp($next, $previous) < 0;
    return $next;
}

sub changelog {
    my ($target, $date, $previous) = @_;
    parts($target);
    parts($previous);
    $date =~ /\A[0-9]{4}-[0-9]{2}-[0-9]{2}\z/ or die "invalid release date\n";
    my $text = read_file('CHANGELOG.md');
    my @sections = $text =~ /^## \[([^\]]+)\](?: - [0-9]{4}-[0-9]{2}-[0-9]{2})?$/mg;
    my %seen;
    for my $section (@sections) {
        die "duplicate changelog section: $section\n" if $seen{$section}++;
        parts($section);
    }
    # Selection and heading rewriting belong to the shared owner. Historical
    # duplicate/version checks above remain an independent corruption boundary.
    # Always carry the saved original version, including after a package bump.
    my $candidate_dir = tempdir("ic-backup-changelog.XXXXXX", TMPDIR => 1, CLEANUP => 0);
    open my $finalizer, "-|", "awk", "-v", "version=$target",
        "-v", "previous=$previous", "-v", "date=$date",
        "-v", "allow_finalized=0", "-f", "$FindBin::Bin/../ci/finalize-release-changelog.awk", "CHANGELOG.md"
        or die "cannot invoke shared changelog finalizer; retained: $candidate_dir\n";
    my $candidate = do { local $/; <$finalizer> };
    my $success = close $finalizer;
    write_file("$candidate_dir/candidate.md", $candidate // "");
    die "shared changelog finalizer failed; retained: $candidate_dir\n" unless $success;
    die "shared changelog finalizer returned no candidate; retained: $candidate_dir\n"
        unless defined($candidate) && length($candidate);
    remove_tree($candidate_dir);
    return $candidate;
}

# Read-only checks share the selected Git source in this process, including resume.
sub check_validation_receipt {
    my ($path, $source, $date, $previous, $candidate, $mode) = @_;
    parts($previous); parts($candidate);
    die "invalid validation source/date\n" unless $source =~ /\A[0-9a-f]{40,64}\z/
        && $date =~ /\A[0-9]{4}-[0-9]{2}-[0-9]{2}\z/;
    my $record = decode_json(read_file($path));
    die "validation identity mismatch\n" unless $record->{schema} == 1
        && $record->{source} eq $source && $record->{date} eq $date
        && $record->{previous} eq $previous && $record->{candidate} eq $candidate
        && $record->{gate} eq 'release-verify';
    die "validation files mismatch\n" unless join(',',sort keys %{$record->{files}})
        eq join(',', receipt_files());
    die "invalid validation check mode\n" unless $mode eq 'original' || $mode eq 'prepared';
    for my $file (keys %{$record->{files}}) {
        next if $mode eq 'prepared' && $file !~ m{\Acrates/[^/]+/Cargo\.toml\z};
        die "validated input changed: $file\n" unless sha256_hex(read_file($file)) eq $record->{files}{$file};
    }
}

sub verify_release_receipt {
    my @selection = @_;
    my $receipt = decode_json(read_file('docs/release.json'));
    die "unsupported release receipt\n" unless $receipt->{schema} == 1;
    die "release receipt selection mismatch\n" if @selection && (@selection != 3
        || $receipt->{source} ne $selection[0] || $receipt->{date} ne $selection[1]
        || $receipt->{version} ne $selection[2]);
    die "release version mismatch\n" unless $receipt->{version} eq version();
    die "invalid release source\n" unless $receipt->{source} =~ /\A[0-9a-f]{40,64}\z/;
    die "invalid release gate\n" unless $receipt->{gate} eq 'release-verify';
    die "invalid release date\n" unless $receipt->{date} =~ /\A[0-9]{4}-[0-9]{2}-[0-9]{2}\z/;
    my @files = sort keys %{$receipt->{files}};
    die "unexpected receipt files\n"
        unless join(',', @files) eq join(',', receipt_files());
    for my $path (@files) {
        die "release file changed after validation: $path\n"
            unless sha256_hex(read_file($path)) eq $receipt->{files}{$path};
    }
    my $notes = read_file('CHANGELOG.md');
    my $heading = "## [$receipt->{version}] - $receipt->{date}";
    die "dated changelog missing\n" unless $notes =~ /^\Q$heading\E$/m;
    return $receipt;
}

my ($command, @args) = @ARGV;
$command //= '';
if (@args >= 2 && $args[-2] eq '--commit') {
    $read_commit = pop @args;
    pop @args;
    die "invalid selected release commit\n" unless $read_commit =~ /\A[0-9a-f]{40,64}\z/;
    die "selected commit only supports read-only checks\n"
        unless $command =~ /\A(?:version|source|verify|validation-check|resume-validation-check)\z/;
}
if ($command eq 'version') {
    my $value = version();
    parts($value);
    print "$value\n";
} elsif ($command eq 'next') {
    print next_version($args[0] // ''), "\n";
} elsif ($command eq 'changelog-check' || $command eq 'finalize') {
    my $text = changelog(@args);
    write_file('CHANGELOG.md', $text) if $command eq 'finalize';
} elsif ($command eq 'prepare-version') {
    my ($target) = @args;
    parts($target);
    my $previous = version();
    my $editor = $ENV{CARGO_RELEASE_METADATA_EDITOR} // '';
    my $limit = $ENV{CARGO_RELEASE_METADATA_MAX_BYTES} // '';
    $editor =~ m{\A/} && -f $editor && -x $editor && $limit =~ /\A[1-9][0-9]*\z/
        or die "select the offline-admitted Cargo editor and positive byte allowance\n";
    my @render = ($editor, '--max-input-bytes', $limit, 'manifest', 'Cargo.toml', $previous, $target);
    # Supply admitted ownership to the common renderer; never duplicate its
    # literal/alias/range policy. Keep both candidates before either write.
    my $manifest_input = read_manifest('Cargo.toml');
    my @members = map { +{
        name => read_manifest($_)->{package}{name},
        manifest_path => decode('UTF-8', abs_path($_) // die("missing workspace member: $_\n"), FB_CROAK | LEAVE_SRC),
    } } member_manifests($manifest_input);
    my $temporary = tempdir('ic-backup-render.XXXXXX', TMPDIR => 1, CLEANUP => 0);
    my $input = "$temporary/workspace-input.json";
    write_file($input, JSON::PP->new->utf8->encode({
        manifest => $manifest_input, members => \@members,
    }));
    push @render, $input;
    open my $rewrite, '-|', $editor, '--max-input-bytes', $limit, 'lock',
        'Cargo.lock', $previous, $target, map { $_->{name} } @members
        or die "cannot invoke shared lockfile transformer; retained: $temporary\n";
    my $lock = do { local $/; <$rewrite> };
    close $rewrite or die "shared lockfile transformer failed; retained: $temporary\n";
    defined($lock) && length($lock) or die "shared lockfile transformer emitted no candidate; retained: $temporary\n";
    my $manifest;
    eval {
        open my $renderer, '-|', @render or die "cannot invoke shared manifest renderer\n";
        $manifest = do { local $/; <$renderer> };
        close $renderer or die "shared manifest renderer failed\n";
        defined($manifest) && length($manifest) or die "shared manifest renderer emitted no candidate\n";
        1;
    } or die $@ . "Manifest inputs retained: $temporary\n";
    remove_tree($temporary);
    write_file('Cargo.toml', $manifest);
    write_file('Cargo.lock', $lock);
} elsif ($command eq 'validation-receipt') {
    my ($path, $source, $date, $previous, $candidate, $mode) = @args;
    parts($previous); parts($candidate);
    die "invalid validation source/date\n" unless $source =~ /\A[0-9a-f]{40,64}\z/
        && $date =~ /\A[0-9]{4}-[0-9]{2}-[0-9]{2}\z/;
    die "validation version changed\n" unless version() eq $previous;
    my %files = map { $_ => sha256_hex(read_file($_)) }
        receipt_files();
    write_file($path, JSON::PP->new->canonical->pretty->encode({
        schema=>1,source=>$source,date=>$date,previous=>$previous,candidate=>$candidate,
        gate=>'release-verify',files=>\%files,
    }));
} elsif ($command eq 'validation-check') {
    check_validation_receipt(@args);
} elsif ($command eq 'resume-validation-check') {
    my ($path, $candidate) = @args;
    parts($candidate);
    my $proof = decode_json(read_file($path));
    my $receipt = decode_json(read_file('docs/release.json'));
    die "resumed validation differs from receipt\n" unless $proof->{candidate} eq $candidate
        && $receipt->{version} eq $candidate && $proof->{source} eq $receipt->{source}
        && $proof->{date} eq $receipt->{date};
    eval {
        check_validation_receipt($path, $proof->{source}, $proof->{date},
            $proof->{previous}, $candidate, 'prepared');
        1;
    } or die $@ . "resumed validation failed\n";
    eval { verify_release_receipt($proof->{source}, $proof->{date}, $candidate); 1 }
        or die $@ . "resumed receipt failed\n";
} elsif ($command eq 'receipt') {
    my ($source, $date) = @args;
    my %hashes = map { $_ => sha256_hex(read_file($_)) }
        receipt_files();
    my $receipt = {
        schema => 1, version => version(), source => $source, date => $date,
        gate => 'release-verify', files => \%hashes,
    };
    write_file('docs/release.json', JSON::PP->new->canonical->pretty->encode($receipt));
} elsif ($command eq 'verify' || $command eq 'source') {
    my $receipt = verify_release_receipt(@args);
    print "$receipt->{source}\n" if $command eq 'source';
} else {
    die "unknown release-data command\n";
}

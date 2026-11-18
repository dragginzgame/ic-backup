#!/usr/bin/env perl
# Backup owns artifact custody; shared tooling owns command capture and redaction.
use strict;
use warnings;
use FindBin;
use lib "$FindBin::Bin/../../vendor/toko-tooling/scripts/ci";
use SetupDiagnostics qw(run_setup);
use File::Path qw(make_path);
use Fcntl qw(O_WRONLY O_CREAT O_EXCL);
my ($phase, @command) = @ARGV;
defined($phase) && $phase =~ /\A[a-z0-9-]+\z/ && @command
    or die "usage: setup.pl PHASE COMMAND [ARG ...]\n";
my $directory = "target/ci-setup/.attempt-$phase";
# CI uses a fresh checkout. Refuse redirected or surviving attempt evidence.
for my $path ('target', 'target/ci-setup', $directory) {
    -l $path and die "refusing redirected setup evidence\n";
}
-e $directory and die "setup evidence already exists: $directory\n";
make_path($directory);
sub collect {
    my ($status, $excerpt) = @_;
    my $summary = "setup phase=$phase tool=$command[0] status=$status\n";
    print STDERR $summary;
    my $collected = eval {
        sysopen my $out, "$directory/diagnostic.txt", O_WRONLY | O_CREAT | O_EXCL, 0600 or die "open\n";
        print {$out} $summary, $excerpt or die "write\n";
        close $out or die "close\n";
        1;
    };
    if (!$collected) {
        print STDERR "setup evidence collection failed; artifact unavailable\n";
        die "diagnostic collection failed\n";
    }
    print STDERR "sanitized evidence: $directory/diagnostic.txt\n";
    if ($status) {
        if (length($excerpt) > 8192) {
            $excerpt = substr($excerpt, -8192);
            $excerpt =~ s/\A[^\n]*\n//;
            print STDERR "diagnostic excerpt truncated to 8 KiB\n";
        }
        print STDERR $excerpt;
    }
}
my $status = run_setup(\&collect, @command);
# Preserve the caller's per-phase success receipt without retaining raw output.
if (!$status) {
    eval { collect(0, "setup command completed\n"); 1 } or $status = 1;
}
exit $status;

#!/usr/bin/env perl
use strict;
use warnings;
use Cwd qw(abs_path);
use File::Temp qw(tempdir);
use File::Path qw(make_path);
my $script = abs_path('scripts/ci/setup.pl');
make_path('target/evidence');
my $root = tempdir('ci-setup-test.XXXXXX', DIR => 'target/evidence', CLEANUP => 0);
chdir $root or die $!;
sub run {
    my ($phase, $code, $expected) = @_;
    system($^X, $script, $phase, $^X, '-e', $code);
    ($? >> 8) == $expected or die "wrong status for $phase: $?\n";
}
$ENV{SETUP_TEST_TOKEN} = 'fixture-secret-987654';
$ENV{SETUP_TEST_ACCESS_KEY} = 'opaque-fixture-3519';
run('success', 'print "progress-sentinel\n"', 0);
open my $success, '<', 'target/ci-setup/.attempt-success/diagnostic.txt' or die $!;
my $receipt = do { local $/; <$success> };
$receipt =~ /status=0/ && $receipt !~ /progress-sentinel/
    or die "success must retain status without raw command output\n";
run('early', 'print STDERR "download failed: checksum mismatch\n$ENV{SETUP_TEST_ACCESS_KEY}\ncontext/$ENV{SETUP_TEST_ACCESS_KEY}/suffix\nAuthorization: Bearer fixture-secret-987654\nhttps://user:password\@host/path?token=abc\nBUILD_HINT=assigned-marker\nhttps://example.invalid/#fragment-marker\naccess\tkey: field-marker\n"; exit 23', 23);
open my $log, '<', 'target/ci-setup/.attempt-early/diagnostic.txt' or die $!;
my $text = do { local $/; <$log> };
$text =~ /checksum mismatch/ && $text =~ /status=23/
    && $text !~ /opaque-fixture-3519|fixture-secret|user:password|token=abc|assigned-marker|fragment-marker|field-marker/ or die "unsafe/missing diagnostic\n";
run('empty', 'exit 19', 19);
run('collector', 'mkdir "target/ci-setup/.attempt-collector/diagnostic.txt"; exit 17', 17);
run('collector-success', 'mkdir "target/ci-setup/.attempt-collector-success/diagnostic.txt"', 1);
run('bounded', 'print "long line\n" x 100000; print "X" x 100000; exit 9', 9);
-s 'target/ci-setup/.attempt-bounded/diagnostic.txt' < 66000 or die "evidence exceeds cap\n";
run('redirected', 'exit 0', 0);
unlink 'target/ci-setup/.attempt-redirected/diagnostic.txt' or die $!;
rmdir 'target/ci-setup/.attempt-redirected' or die $!;
symlink '/tmp', 'target/ci-setup/.attempt-redirected' or die $!;
system($^X, $script, 'redirected', $^X, '-e', 'exit 0');
($? >> 8) != 0 or die "followed redirected evidence\n";
print "setup failure, hidden evidence, redaction, caps, empty output, collection failure and symlink checks pass\n";

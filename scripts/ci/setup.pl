#!/usr/bin/env perl
# Capture only the selected public-tool setup command, never a workspace/cache.
use strict;
use warnings;
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
my @secrets = grep { length($_) >= 6 } map { $ENV{$_} }
    grep { /token|password|secret|credential|authorization|private.key/i } keys %ENV;
my ($tail, $pending, $truncated, $omitted, $private) = ('', '', 0, 0, 0);
sub retain {
    my ($line) = @_;
    if ($line =~ /-----BEGIN .*PRIVATE KEY-----/) { $private = 1; }
    if ($private) {
        $private = 0 if $line =~ /-----END .*PRIVATE KEY-----/;
        $line = "[private key material omitted]\n";
    }
    for my $secret (@secrets) { $line =~ s/\Q$secret\E/[REDACTED]/g; }
    $line =~ s/\e\[[0-?]*[ -\/]*[\@-~]//g;
    $line =~ s/[\x00-\x08\x0b-\x1f\x7f]/?/g;
    $line =~ s{https?://[^\s<>"']+}{[URL omitted]}gi;
    $line =~ s/(?:gh[pousr]_|github_pat_)[A-Za-z0-9_]+/[REDACTED]/g;
    if ($line =~ /(?:token|password|secret|credential|authorization|private.key)\s*[=:]/i) {
        $line = "[sensitive diagnostic line omitted]\n";
    }
    $tail .= $line;
    if (length($tail) > 65536) {
        # Trim only at line boundaries so redacted fields cannot become fragments.
        substr($tail, 0, index($tail, "\n") + 1, '') while length($tail) > 65536;
        $truncated = 1;
    }
}
my $pid = open(my $pipe, '-|');
defined $pid or die "cannot start setup command: $!\n";
if (!$pid) {
    open STDERR, '>&', STDOUT or exit 126;
    exec {$command[0]} @command or exit 127;
}
my $discard = 0;
while (1) {
    my $count = sysread($pipe, my $chunk, 4096);
    defined $count or die "cannot read setup diagnostics: $!\n";
    last unless $count;
    $pending .= $chunk;
    while ($pending =~ s/\A([^\n]*\n)//) {
        my $line = $1;
        if (!$discard && length($line) <= 16384) { retain($line); }
        else { $omitted = 1; }
        $discard = 0;
    }
    if (length($pending) > 16384) { $pending = ''; $discard = 1; $omitted = 1; }
}
retain("$pending\n") if length($pending) && !$discard;
close $pipe;
my $wait = $?;
my $status = $wait == -1 ? 125 : ($wait & 127) ? 128 + ($wait & 127) : $wait >> 8;
my $summary = "setup phase=$phase tool=$command[0] status=$status\n";
$summary .= "diagnostics truncated to last 64 KiB\n" if $truncated;
$summary .= "overlong diagnostic lines omitted\n" if $omitted;
$summary .= "command emitted no diagnostics\n" unless length($tail);
my $collected = eval {
    sysopen my $out, "$directory/diagnostic.txt", O_WRONLY | O_CREAT | O_EXCL, 0600 or die "open: $!";
    print {$out} $summary, $tail or die "write: $!";
    close $out or die "close: $!";
    1;
};
print STDERR $summary;
if ($collected) { print STDERR "sanitized evidence: $directory/diagnostic.txt\n"; }
else { print STDERR "setup evidence collection failed; artifact unavailable\n"; }
if ($status) {
    my $excerpt = $tail;
    if (length($excerpt) > 8192) {
        $excerpt = substr($excerpt, -8192);
        $excerpt =~ s/\A[^\n]*\n//;
        print STDERR "diagnostic excerpt truncated to 8 KiB\n";
    }
    print STDERR $excerpt;
}
exit($status || ($collected ? 0 : 1));

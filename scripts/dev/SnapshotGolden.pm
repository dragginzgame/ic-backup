package SnapshotGolden;
use strict;
use warnings;
use Exporter 'import';
use JSON::PP;
use Math::BigInt;

our @EXPORT_OK = qw(uleb sleb field_hash labels fields variant argument write_json);

# Independent golden-vector encoding primitives; production Rust never uses these.
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
    my ($tag, $values) = @_;
    return sleb($tag) . uleb(scalar keys %$values) . join('',
        map { uleb(field_hash($_)) . sleb($values->{$_}) } labels($values));
}

sub variant {
    my ($values, $name, $value) = @_;
    my @labels = labels($values);
    my ($index) = grep { $labels[$_] eq $name } 0..$#labels;
    return uleb($index) . $value;
}

sub argument {
    my ($types, $fields, $values) = @_;
    return 'DIDL' . uleb(scalar @$types) . join('', @$types) . uleb(1) . sleb(0)
        . join('', map { $values->{$_} } labels($fields));
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

1;

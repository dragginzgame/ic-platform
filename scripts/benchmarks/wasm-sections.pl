#!/usr/bin/env perl
# One framing reader for total, payload and framed Wasm-size measurements.
use strict;
use warnings;
use JSON::PP;
use Digest::SHA qw(sha256_hex);
my $json = @ARGV && $ARGV[0] eq '--json' ? shift(@ARGV) : '';
@ARGV == 1 or die "usage: wasm-sections.pl [--json] MODULE|-\n";
my $input;
if ($ARGV[0] eq '-') { binmode STDIN; $input = \*STDIN; }
else { open $input, '<:raw', $ARGV[0] or die "$ARGV[0]: $!\n"; }
local $/; my $bytes = <$input>; close $input or die $!;
substr($bytes,0,8) eq "\0asm\x01\0\0\0" or die "not Wasm v1\n";
my ($offset,$code,$framed,$custom) = (8,0,0,0);
my @sections;
while ($offset < length($bytes)) {
    my $start = $offset;
    my $id = ord(substr($bytes,$offset++,1));
    my ($size,$done) = (0,0);
    for my $index (0..4) {
        $offset < length($bytes) or die "truncated section length\n";
        my $byte = ord(substr($bytes,$offset++,1));
        $index != 4 || $byte <= 15 or die "section length exceeds u32\n";
        $size |= ($byte & 127) << ($index * 7);
        if (!($byte & 128)) { $done=1; last; }
    }
    $done && $size <= length($bytes)-$offset or die "bad section length\n";
    my $framed_size = $offset-$start+$size;
    push @sections, {id=>$id,payloadBytes=>$size,framedBytes=>$framed_size,
        sha256=>sha256_hex(substr($bytes,$offset,$size))};
    $code += $size if $id == 10;
    $framed += $framed_size if $id == 10;
    $custom += $size if $id == 0;
    $offset += $size;
}
if ($json) {
    print JSON::PP->new->canonical->encode({totalBytes=>length($bytes),codePayloadBytes=>$code,
        codeFramedBytes=>$framed,customPayloadBytes=>$custom,sha256=>sha256_hex($bytes),sections=>\@sections}),"\n";
} else { print length($bytes),"\t",$code,"\t",$custom; }

use strict; use warnings;
open my $input, '<:raw', $ARGV[0] or die $!;
local $/; my $bytes = <$input>;
die 'not Wasm' unless substr($bytes, 0, 4) eq "\0asm";
my ($offset, $code, $custom) = (8, 0, 0);
while ($offset < length($bytes)) {
    my $id = ord(substr($bytes, $offset++, 1));
    my ($size, $shift, $byte) = (0, 0, 0);
    do { $byte = ord(substr($bytes, $offset++, 1)); $size |= ($byte & 127) << $shift; $shift += 7; } while ($byte & 128);
    $code += $size if $id == 10;
    $custom += $size if $id == 0;
    $offset += $size;
}
die 'bad section length' unless $offset == length($bytes);
print length($bytes), "\t", $code, "\t", $custom;

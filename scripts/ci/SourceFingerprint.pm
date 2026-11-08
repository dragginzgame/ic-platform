package SourceFingerprint;
use strict;
use warnings;
use Exporter 'import';
our @EXPORT_OK = qw(normalize);

# Preserve the exact historical fingerprint recipe and literal contents.
# Callers still select/reconstruct their own protected source regions.
sub normalize {
    my ($source) = @_;
    my @strings;
    $source =~ s{"(?:\\.|[^"\\])*"}{push(@strings, $&); "\x01" . $#strings . "\x02"}gse;
    $source =~ s{//[^\n]*}{}g;
    $source =~ s{/\*.*?\*/}{}gs;
    $source =~ s/\s+//g;
    $source =~ s/\x01(\d+)\x02/$strings[$1]/ge;
    return $source;
}
1;

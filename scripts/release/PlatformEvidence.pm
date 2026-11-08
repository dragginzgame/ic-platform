package PlatformEvidence;
use strict;
use warnings;
use File::Basename qw(dirname);
use lib dirname(__FILE__) . '/../ci';
use ValidationPlan ();
use Exporter 'import';
our @EXPORT_OK = qw(bytes digest command);

sub route {
    my ($path) = @_;
    defined($path) && !ref($path) or die "invalid evidence path\n";
    return $path =~ m{^/} ? ('/', substr($path, 1)) : ('.', $path);
}
# Preserve the existing native evidence size contract rather than adopting the
# shared helpers' smaller document defaults. Digests never retain file contents.
sub bytes { ValidationPlan::bytes(route($_[0]), ~0) }
sub digest { ValidationPlan::digest(route($_[0]), ~0) }
sub command {
    open my $in, '-|', @_ or die "execute @_: $!\n";
    local $/; my $value = <$in> // '';
    close $in or die "command failed: @_\n";
    return $value;
}
1;

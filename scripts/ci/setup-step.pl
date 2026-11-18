use strict;
use warnings;
use FindBin;
use lib "$FindBin::Bin/../../vendor/toko-tooling/scripts/ci";
use SetupDiagnostics qw(run_setup);
use File::Temp qw(tempfile);

my $phase = shift;
exit 64 unless defined $phase && $phase =~ /\A[a-z0-9-]+\z/ && @ARGV;
my $directory = 'target/ci-setup';
for my $path ('target', $directory) {
    if (-l $path || (!-d $path && !mkdir($path) && !-d $path)) {
        warn "Unsafe or unavailable setup evidence directory\n";
        exit 73;
    }
}

my $status = run_setup(sub {
    my ($code, $excerpt) = @_;
    my $output = "$directory/$phase.log";
    if (-l $output) {
        warn "Evidence collection refused symlink; original setup status: $code\n";
        return;
    }
    # Retain Platform's suppression of bearer lines and relative URL queries.
    $excerpt =~ s/^.*(?:bearer|private.?key).*$/[sensitive line omitted]/mig;
    $excerpt =~ s/\?[^\s]*/?/g;
    my $report = "Setup phase: $phase; original exit status: $code\n$excerpt";
    my ($file, $temp) = tempfile('setup-XXXXXXXX', DIR => $directory, UNLINK => 1);
    print {$file} $report or die "cannot write setup evidence: $!";
    close $file or die "cannot close setup evidence: $!";
    rename $temp, $output or die "cannot install setup evidence: $!";
    print "Setup failed; sanitized evidence: $output\n", $report;
}, @ARGV);
print "Setup $phase succeeded\n" unless $status;
exit $status;

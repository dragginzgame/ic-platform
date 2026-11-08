#!/usr/bin/env perl
# Execution identities used by both fresh validation and recovery admission.
use strict;
use warnings;
use JSON::PP;
use Digest::SHA;
use Cwd qw(abs_path);
use FindBin;
use lib $FindBin::Bin;
use PlatformEvidence qw(command);
sub fingerprint {
    my ($path)=@_; chomp $path;
    my $resolved=abs_path($path);
    defined($resolved) && -f $resolved && -x $resolved or die "missing executable: $path\n";
    open my $fh,'<:raw',$resolved or die "read executable: $!\n";
    return {path=>$resolved,sha256=>Digest::SHA->new(256)->addfile($fh)->hexdigest};
}
my %tools;
for my $tool (qw(rustc cargo)) {
    $tools{$tool}=fingerprint(command('bash','-c','command -v "$1"','tool',$tool));
    $tools{"$tool-toolchain"}=fingerprint(command('rustup','which',$tool));
}
$tools{server}=fingerprint($ENV{POCKET_IC_BIN}) if length($ENV{POCKET_IC_BIN}//'');
my $host=command('uname','-sm');
my $host_key=$host; chomp $host_key;
if ($host_key =~ /^Darwin /) {
    my $version=command('sw_vers','-productVersion');
    $version =~ /\A(\d+)\./ or die "unknown macOS identity\n";
    $host_key.=" $1";
}
my $context={host=>$host,host_key=>$host_key,rustc=>command('rustc','-vV'),cargo=>command('cargo','-V'),
    tools=>\%tools,environment=>{map {$_=>(exists($ENV{$_})?$ENV{$_}:undef)}
        qw(PATH RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTDOCFLAGS CARGO_TARGET_DIR RUSTUP_TOOLCHAIN CC CXX AR POCKET_IC_BIN IC_TESTKIT_POCKET_IC_URL)} };
print JSON::PP->new->canonical->utf8->encode($context),"\n";

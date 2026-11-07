#!/usr/bin/env perl
# Compose Shared's unchanged stage admissions into Platform's complete receipt.
use strict;
use warnings;
use JSON::PP;
use Cwd qw(abs_path);
use File::Path qw(make_path);
my $json=JSON::PP->new->canonical->utf8;
sub command {
    open my $in,'-|',@_ or die "execute @_: $!\n";
    local $/; my $raw=<$in>//''; close $in or die "failed @_; no policy-fixture admission\n"; return $raw;
}
sub environment {
    # Private fixture Git configuration prevents ambient hooks, signing or
    # templates. These tests invoke compiler version queries, not package builds.
    -d 'target' && !-l 'target' or die "unsafe fixture target\n";
    my $scratch=abs_path('target') . '/policy-fixture-environment';
    make_path("$scratch/cargo","$scratch/templates","$scratch/bin");
    for my $path ($scratch,"$scratch/cargo","$scratch/templates","$scratch/bin") { !-l $path or die "redirected fixture environment\n"; }
    for my $path ("$scratch/.gitconfig","$scratch/cargo/config","$scratch/cargo/config.toml") {
        !-e $path && !-l $path or die "ambient fixture configuration\n";
    }
    for my $directory ("$scratch/cargo","$scratch/templates") {
        opendir my $contents,$directory or die $!;
        !grep {$_ ne '.' && $_ ne '..'} readdir $contents or die "ambient fixture environment files\n";
        closedir $contents or die $!;
    }
    # A closed PATH makes a newly introduced external command fail rather than
    # executing an unobserved ambient tool. Preserve the caller's actual tool
    # selection and bind its original PATH too; never substitute another tool.
    my @names=qw(bash perl jq git make tar sed grep cp mv rm mkdir chmod cat cmp head tail sort find date sleep touch ln pwd cut readlink wc tr env uname rustc cargo rustup sh dirname basename mktemp rmdir awk tee sha256sum sha512sum printf echo true false test [);
    my %names=map {$_=>1} @names;
    opendir my $bin,"$scratch/bin" or die $!;
    for (grep {$_ ne '.' && $_ ne '..'} readdir $bin) { $names{$_} && -l "$scratch/bin/$_" or die "unexpected fixture tool\n"; }
    closedir $bin or die $!;
    for my $name (@names) {
        my ($selected)=grep {-f $_ && -x $_} map {"$_/$name"} split /:/,$ENV{PATH};
        defined($selected) or die "missing fixture tool $name\n";
        # Keep the invocation basename for rustup's cargo/rustc dispatchers.
        my $path="$scratch/bin/$name";
        if (-l $path && readlink($path) ne $selected) { unlink $path or die $!; }
        symlink($selected,$path) or die $! unless -l $path;
    }
    my $rustup=command('rustup','show','home'); chomp $rustup;
    return {PATH=>"$scratch/bin",PLATFORM_POLICY_TOOL_PATH=>$ENV{PATH},HOME=>$scratch,CARGO_HOME=>"$scratch/cargo",RUSTUP_HOME=>$rustup,
        RUSTUP_TOOLCHAIN=>'1.99.1',RUSTUP_AUTO_INSTALL=>'0',CARGO_NET_OFFLINE=>'true',LC_ALL=>'C',TZ=>'UTC',
        GIT_CONFIG_NOSYSTEM=>'1',GIT_CONFIG_GLOBAL=>'/dev/null',GIT_ATTR_NOSYSTEM=>'1',
        GIT_TEMPLATE_DIR=>"$scratch/templates"};
}
my ($mode,@args)=@ARGV;
if (($mode//'') eq 'environment' && @args==1) {
    !-e $args[0] && !-l $args[0] or die "occupied stage environment\n";
    open my $out,'>:raw',$args[0] or die $!; print {$out} $json->encode(environment()),"\n"; close $out or die $!;
    exit;
}
($mode//'') eq 'seal' && @args==3 or die "usage: policy-fixture-evidence.pl environment OUTPUT | seal ADMISSION SOURCE GROUP\n";
my ($path,$source,$group)=@args;
my $env=environment();
my $observation;
{ local %ENV=%$env; $observation=$json->decode(command('scripts/release/observe-policy-fixtures.pl')); }
my $sealed=$json->decode(command('perl','scripts/ci/seal-validation-stage.pl',abs_path($path)//$path,$source,$group,$observation->{configuration}{identity}));
$sealed->{admission}{stage} eq 'release-policy-fixtures' && $sealed->{admission}{contract} eq 'platform-policy-fixtures/1'
    && $sealed->{cwd} eq abs_path('.') or die "wrong policy-fixture stage\n";
print $json->encode($sealed),"\n";

#!/usr/bin/env perl
# Closed, non-Cargo policy-fixture inputs; Linux execution only (contract /1).
use strict;
use warnings;
use Cwd qw(abs_path);
use Digest::SHA qw(sha256_hex);
use File::Find;
use JSON::PP;
use DynaLoader ();
use POSIX qw(uname);
use FindBin;
use lib $FindBin::Bin;
use PlatformEvidence qw(command);
my $json=JSON::PP->new->canonical->utf8;
my @host=uname();
$host[0] eq 'Linux' && $host[4] eq 'x86_64' or die "policy-fixture reuse is qualified only on Linux x86_64\n";
my %files;
# These fixtures copy policy scripts/companions, extract Make recipes and create
# their own package/version/lock files. They never read the real Cargo graph,
# release ledgers or maintained installed-runtime locks. Enumerate additions too.
for my $directory (qw(scripts/ci scripts/release docs rules make)) {
    find({no_chdir=>1,wanted=>sub {
        -l $_ and die "redirected policy input $_\n";
        $files{$_}=1 if -f $_;
    }},$directory);
}
$files{$_}=1 for qw(Makefile .shared-tooling.snapshot DRAGGINZGAME.md AGENTS.md README.md rust-toolchain.toml .cargo/config.toml ci/release-validation.json ci/cargo-release-groups.json);
my $bin=$ENV{PATH};
$bin =~ m{\A/} && $bin !~ /:/ && -d $bin or die "policy fixtures require their closed tool PATH\n";
opendir my $directory,$bin or die $!;
my @tools=sort grep {$_ ne '.' && $_ ne '..'} readdir $directory;
closedir $directory or die $!;
my %tools;
for my $name (@tools) {
    my $path="$bin/$name";
    -l $path && -f $path && -x $path or die "missing policy tool $name\n";
    $files{$path}=1; $tools{$name}=abs_path($path);
}
for my $path (qw(/usr/bin/env /bin/sh /bin/bash)) { $files{$path}=1; $tools{$path}=abs_path($path); }
for my $name (qw(rustc cargo)) { my $path=command('rustup','which',$name); chomp $path; $files{$path}=1; $tools{"$name-toolchain"}=$path; }
# Local pushes call Git's private helpers. Perl scripts may load further core
# modules after observation. Bind the expanded installed runtimes, including
# additions, rather than guessing the modules/helpers used on a success path.
my $git_helpers=command('git','--exec-path'); chomp $git_helpers;
my %runtime_directories=map {abs_path($_)=>1} grep {-d $_} (@INC,$git_helpers);
for my $directory (sort keys %runtime_directories) {
    find({no_chdir=>1,follow=>1,follow_skip=>2,wanted=>sub {
        $files{abs_path($_)}=1 if -f $_;
    }},$directory);
}
# Bind runtime libraries as well as executable bytes. ldd is an observation tool,
# never a command on an untrusted input; all paths above are selected host tools.
$files{'/usr/bin/ldd'}=1;
for my $tool (values %tools) {
    open my $binary,'<:raw',$tool or die $!;
    read($binary,my $magic,4)==4 or die "unreadable tool header\n";
    close $binary or die $!;
    next unless $magic eq "\x7fELF";
    my $child=open my $in,'-|'; defined($child) or die $!;
    if (!$child) {
        open STDERR,'>&',\*STDOUT or die $!;
        exec '/usr/bin/ldd',$tool or die $!;
    }
    local $/; my $libraries=<$in>//''; my $success=close $in;
    $success || $libraries =~ /not a dynamic executable|statically linked/
        or die "tool library observation failed: $tool\n";
    $libraries !~ /not found/ or die "missing tool library: $tool\n";
    $files{$1}=1 while $libraries =~ m{(/[^\s()]+)}g;
}
$files{abs_path($_)}=1 for values %INC, @DynaLoader::dl_shared_objects;
my @inventory;
for my $path (sort keys %files) {
    -f $path or die "missing policy input $path\n";
    open my $in,'<:raw',$path or die "read $path: $!\n";
    push @inventory,{path=>$path,resolved=>abs_path($path),mode=>0+(stat($path))[2],sha256=>Digest::SHA->new(256)->addfile($in)->hexdigest};
    close $in or die $!;
}
my $compiler={rustc=>command('rustc','-vV'),cargo=>command('cargo','-V')};
my $configuration={host=>\@host,compiler=>$compiler,tools=>\%tools,environment=>{%ENV},perl_inc=>\@INC,git_helpers=>$git_helpers};
my $identity=sha256_hex($json->encode({configuration=>$configuration,files=>\@inventory}));
print $json->encode({schema=>1,files=>[sort keys %files],configuration=>{%$configuration,identity=>$identity}}),"\n";

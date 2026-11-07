#!/usr/bin/env perl
# Consumer evidence admission; Shared retains release/source/metadata custody.
use strict;
use warnings;
use JSON::PP;
use Digest::SHA qw(sha256_hex);
use Cwd qw(abs_path);
use File::Basename qw(dirname);
use File::Temp qw(tempdir tempfile);
use File::Path qw(make_path);
my $json=JSON::PP->new->canonical->utf8;
my ($plan_path,$run,$output,$mode)=@ARGV;
$mode //= 'write';
(@ARGV==3 || @ARGV==4) && $mode =~ /\A(?:write|check)\z/
    or die "usage: validation-receipt.pl PLAN RUN-DIRECTORY RECEIPT [write|check]\n";
sub bytes {
    my ($path)=@_;
    $path !~ m{[\x00-\x1f]|(?:^|/)\.\.(?:/|$)} or die "unsafe evidence path\n";
    my $route=$path =~ m{^/}?'/':'.';
    for (grep {length} split m{/},$path) { $route.="/$_"; !-l $route or die "redirected evidence\n"; }
    -f $path or die "missing evidence $path\n";
    open my $fh,'<:raw',$path or die "read $path: $!\n";
    local $/; return <$fh>;
}
sub command {
    open my $fh,'-|',@_ or die "execute @_: $!\n";
    local $/; my $value=<$fh>//''; close $fh or die "failed @_; no receipt\n"; return $value;
}
my $raw=bytes($plan_path);
my $plan=$json->decode($raw);
my $context=$json->decode(bytes("$plan_path.context.json"));
my $schema=exists($context->{stage_admissions})?4:3;
$plan->{scope} eq "workspace" && !grep { !$_->{required} } @{$plan->{stages}}
    or die "Platform requires the full validation gate\n";
my $checkpoint_root=abs_path('.');
my $physical_root=$checkpoint_root;
my $projected=0;
if ($mode eq 'write') {
    command('git','rev-parse','HEAD') eq "$plan->{source}\n" or die "stale source\n";
    command('git','status','--porcelain','--untracked-files=all') eq '' or die "dirty release inputs\n";
    command('git','ls-files','--stage','-z') eq $context->{index_inventory} or die "source inventory changed\n";
} else {
    defined($context->{release}) or die "qualification receipt cannot admit a release\n";
    ($ENV{RELEASE_SOURCE}//'') eq $plan->{source} && ($ENV{RELEASE_GROUP}//'') eq $plan->{group}
        or die "wrong release source/group\n";
    my %allowed=map {$_=>1} grep {length} split /\0/, command('bash','scripts/release/group.sh','files');
    # Shared's metadata editor remains the sole version projector. Only its
    # declared files may differ; the adapter still checks their exact semantics.
    for my $staged (0,1) {
        my @args=('git','diff','--name-only','--no-renames','-z');
        push @args,'--cached' if $staged;
        for (grep {length} split /\0/,command(@args,$plan->{source},'--')) {
            $allowed{$_} or die "non-metadata source drift: $_\n";
            $projected=1;
        }
    }
    command('git','ls-files','--others','--exclude-standard','-z') eq '' or die "untracked release source\n";
    my $inventory='';
    for (grep {length} split /\0/,command('git','ls-tree','-rz','--full-tree',$plan->{source})) {
        /\A(\d+) (?:blob|commit) ([0-9a-f]+)\t(.+)\z/s or die "invalid source tree\n";
        $inventory.="$1 $2 0\t$3\0";
    }
    $inventory eq $context->{index_inventory} or die "receipt source inventory mismatch\n";
    # Validate original tracked inputs from their immutable Git source, not a
    # hand-written reversal of prepared Cargo metadata. The live diff above
    # preserves all non-metadata obligations, including added/staged files.
    $checkpoint_root=tempdir('platform-validation-source.XXXXXX',TMPDIR=>1,CLEANUP=>1);
    system('bash','-euo','pipefail','-c','git archive "$1" | tar -xf - -C "$2"',
        'validation-source',$plan->{source},$checkpoint_root)==0 or die "cannot reconstruct validated source\n";
    for my $path ($plan_path,"$plan_path.context.json") {
        $path =~ m{\Atarget/[A-Za-z0-9_./-]+\z} && $path !~ m{(?:^|/)\.\.?/} or die "unsafe retained plan\n";
        my $value=bytes($path);
        make_path(dirname("$checkpoint_root/$path"));
        open my $file,'>:raw',"$checkpoint_root/$path" or die $!;
        print {$file} $value; close $file or die $!;
    }
}
if (defined $context->{release}) {
    my $release={map {$_=>$ENV{'RELEASE_'.uc($_)}} qw(previous version remote branch)};
    $json->encode($release) eq $json->encode($context->{release}) or die "release identity/destination drift\n";
}
my $runner_log=dirname(dirname($run)).'/runner.log';
my $runner_bytes=bytes($runner_log);
$runner_bytes =~ /\nPlatform runner completed status=0\n\z/
    or die "runner did not finish successfully\n";
$context->{source} eq $plan->{source} && $context->{group} eq $plan->{group} or die "wrong context\n";
if ($context->{baseline_tag} ne 'none') {
    command('git','rev-parse',"refs/tags/$context->{baseline_tag}^{commit}") eq "$context->{baseline}\n"
        or die "baseline moved\n";
}
my @verify=('perl','scripts/ci/validation-evidence.pl',
    $physical_root,$plan_path,$plan->{source},$plan->{group},$run,"$plan_path.context.json");
my $proof;
if ($mode eq 'write') {
    # Shared alone interprets events, timings, stage checkpoints and log slots.
    # Its nonzero status must never publish a successful consumer receipt.
    $proof=$json->decode(command($verify[0],$verify[1],'verify',@verify[2..$#verify]));
} else {
    my $saved=$json->decode(bytes($output));
    $saved->{schema}==$schema && $saved->{physical_root} eq $physical_root
        or die "unsupported or relocated consumer receipt; restore its original source/location\n";
    $proof=$saved->{validation_evidence};
    ref($proof) eq 'HASH' or die "missing complete Shared proof\n";
    if (!$projected) {
        # Re-admit at the original physical source/run location. Do not rewrite
        # the runner's absolute paths or claim that this checks a projection.
        my ($fh,$path)=tempfile('receipt-proof.XXXXXX',DIR=>$run,UNLINK=>1);
        print {$fh} $json->encode($proof),"\n"; close $fh or die $!;
        command($verify[0],$verify[1],'check',@verify[2..$#verify],$path);
    } else {
        # Shared cannot read prepared metadata as the original source. The
        # consumer gate protects this complete receipt by digest in its journal.
        # Re-admit immutable source inputs through the existing checkpoint owner,
        # and bind every retained evidence file without parsing runner protocols.
        my ($fh,$observed)=tempfile('receipt-inputs.XXXXXX',DIR=>$run,UNLINK=>1); close $fh;
        system('perl','scripts/ci/validation-input-checkpoint.pl',$checkpoint_root,"$run/plan-inputs.json",$observed,
            'scripts/ci/verify-file-checksum.sh',@{$plan->{inputs}})==0 or die "input checkpoint drift\n";
        $json->encode($proof->{inputs}) eq $json->encode($json->decode(bytes("$run/plan-inputs.json")))
            or die "Shared proof input inventory drift\n";
        for my $path (keys %{$proof->{files}}) {
            sha256_hex(bytes("$run/$path")) eq $proof->{files}{$path} or die "retained Shared evidence changed: $path\n";
        }
    }
}
my $execution=$json->decode(command('perl','scripts/release/validation-context.pl'));
for (keys %$execution) {
    $json->encode($execution->{$_}) eq $json->encode($context->{$_}) or die "execution context drift: $_\n";
}
my %stage_admissions;
for my $stage (@{$context->{stage_admissions}//[]}) {
    $stage eq 'release-policy-fixtures' or die "unqualified reusable stage\n";
    my $path=dirname(dirname($run))."/stage-admissions/$stage.json";
    bytes($path); # Check routes before the Shared sealer reads the admission.
    # The leaf was dispatched by a child Make, whose exported tool PATH is a
    # genuine input. Re-enter the same Make owner for observation; the parent's
    # environment is not interchangeable and must not be normalized for reuse.
    local $ENV{PLATFORM_POLICY_ADMISSION}=$path;
    local $ENV{PLATFORM_POLICY_SOURCE}=$plan->{source};
    local $ENV{PLATFORM_POLICY_GROUP}=$plan->{group};
    $stage_admissions{$stage}=$json->decode(command('make','--no-print-directory','check-policy-fixture-evidence'));
}
my $receipt={schema=>$schema,contract=>"platform-release-validation/$schema",source=>$plan->{source},group=>$plan->{group},
    scope=>$plan->{scope},baseline=>$context->{baseline},plan_sha256=>sha256_hex($raw),
    context_sha256=>sha256_hex(bytes("$plan_path.context.json")), release=>$context->{release},
    policy_sha256=>sha256_hex(bytes($plan->{policy})),inputs_sha256=>sha256_hex(bytes("$run/plan-inputs.json")),
    %$execution,runner_log_sha256=>sha256_hex(bytes($runner_log)),
    physical_root=>$physical_root,validation_evidence=>$proof,
    release_omission_admitted=>JSON::PP::false};
$receipt->{stage_admissions}=\%stage_admissions if $schema==4;
if ($mode eq 'check') {
    $json->encode($receipt)."\n" eq bytes($output) or die "receipt/evidence changed\n";
    print "Release validation receipt admitted: $output\n";
    exit 0;
}
!-e $output && !-l $output or die "never overwrite a receipt\n";
open my $out,'>:raw',$output or die "write receipt: $!\n";
print {$out} $json->encode($receipt),"\n"; close $out or die "close receipt\n";
print "$output\n";

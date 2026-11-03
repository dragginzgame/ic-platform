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
$raw eq bytes("$run/parsed-inputs.json.plan.json") or die "runner selected a different plan\n";
bytes($plan->{policy}) eq bytes("$run/parsed-inputs.json.policy.json") or die "policy drift\n";
my $context=$json->decode(bytes("$plan_path.context.json"));
my $checkpoint_root=abs_path('.');
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
$runner_bytes =~ /^VALIDATION PASSED: all requested targets succeeded\.$/m
    && $runner_bytes =~ /\nPlatform runner completed status=0\n\z/
    or die "runner did not finish successfully\n";
$context->{source} eq $plan->{source} && $context->{group} eq $plan->{group} or die "wrong context\n";
if ($context->{baseline_tag} ne 'none') {
    command('git','rev-parse',"refs/tags/$context->{baseline_tag}^{commit}") eq "$context->{baseline}\n"
        or die "baseline moved\n";
}
# Reuse Shared's checkpoint validator; do not implement a second source hasher.
my ($observed_fh,$observed)=tempfile('receipt-inputs.XXXXXX',DIR=>$run); close $observed_fh;
system('perl','scripts/ci/validation-input-checkpoint.pl',$checkpoint_root,"$run/plan-inputs.json",$observed,
    'scripts/ci/verify-file-checksum.sh',@{$plan->{inputs}})==0 or die "input checkpoint drift\n";
my %last;
for my $line (split /\n/,bytes("$run/plan-events.tsv")) {
    my ($target,$state,$detail)=split /\t/,$line,3;
    next if $target eq 'target';
    $last{$target}=[$state,$detail];
}
my %timings;
for my $line (split /\n/,bytes("$run/timings.tsv")) {
    my ($target,$state,$seconds,$log)=split /\t/,$line,4;
    next if $target eq 'target';
    !exists $timings{$target} && $state eq 'PASS' && $seconds =~ /^\d+$/
        or die "failed/duplicate/incomplete stage timing\n";
    $timings{$target}=[$seconds,$log];
}
my @stages;
for my $stage (@{$plan->{stages}}) {
    my $target=$stage->{target}; my $event=$last{$target} or die "missing stage event\n";
    if ($stage->{required}) {
        $event->[0] eq 'PASS' && exists $timings{$target} && $event->[1] eq $timings{$target}[1]
            or die "blocked, failed or missing required stage $target\n";
        my $log=$event->[1];
        index(abs_path($log)//'',abs_path($run).'/')==0 or die "stage log outside attempt\n";
        push @stages,{target=>$target,state=>'PASS',admission=>'executed',seconds=>0+$timings{$target}[0],
            log=>$log,log_sha256=>sha256_hex(bytes($log))};
    } else {
        $event->[0] eq 'not-applicable' && !exists $timings{$target} or die "excluded stage executed\n";
        push @stages,{target=>$target,state=>'not-applicable',reason=>$stage->{reason}};
    }
    delete $last{$target}; delete $timings{$target};
}
!keys(%last) && !keys(%timings) or die "unknown evidence stage\n";
my $execution=$json->decode(command('perl','scripts/release/validation-context.pl'));
for (keys %$execution) {
    $json->encode($execution->{$_}) eq $json->encode($context->{$_}) or die "execution context drift: $_\n";
}
my $receipt={schema=>2,contract=>'platform-release-validation/2',source=>$plan->{source},group=>$plan->{group},
    scope=>$plan->{scope},baseline=>$context->{baseline},plan_sha256=>sha256_hex($raw),
    context_sha256=>sha256_hex(bytes("$plan_path.context.json")), release=>$context->{release},
    policy_sha256=>sha256_hex(bytes($plan->{policy})),inputs_sha256=>sha256_hex(bytes("$run/plan-inputs.json")),
    %$execution,runner_log_sha256=>sha256_hex(bytes($runner_log)),stages=>\@stages,
    release_omission_admitted=>(defined($context->{release}) && $plan->{scope} eq 'group')?JSON::PP::true:JSON::PP::false};
if ($mode eq 'check') {
    $json->encode($receipt)."\n" eq bytes($output) or die "receipt/evidence changed\n";
    print "Release validation receipt admitted: $output\n";
    exit 0;
}
!-e $output && !-l $output or die "never overwrite a receipt\n";
open my $out,'>:raw',$output or die "write receipt: $!\n";
print {$out} $json->encode($receipt),"\n"; close $out or die "close receipt\n";
print "$output\n";

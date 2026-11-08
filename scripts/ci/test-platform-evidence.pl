#!/usr/bin/env perl
use strict;
use warnings;
use FindBin;
use lib "$FindBin::Bin/../release";
use PlatformEvidence qw(bytes digest command);
use File::Temp qw(tempdir);
use Digest::SHA qw(sha256_hex);
use POSIX ();
use Cwd qw(getcwd);
use Test::More;
chdir "$FindBin::Bin/../.." or die $!;
mkdir 'target' unless -d 'target';
my $root = tempdir('platform-evidence.XXXXXX', DIR=>'target', CLEANUP=>1);
my $path = "$root/large";
my $value = 'e' x (1048576 + 1);
open my $out, '>:raw', $path or die $!;
print {$out} $value; close $out or die $!;
is(bytes($path), $value, 'existing evidence larger than shared document default remains admitted');
is(digest($path), sha256_hex($value), 'streamed digest preserves exact bytes');
is(bytes(getcwd()."/$path"), $value, 'absolute retained evidence uses the same reader');
is(command($^X, '-e', 'print "ok"'), 'ok', 'successful command retains exact output');
is(command($^X, '-e', 'exit 0'), '', 'successful empty command stays empty');
sub refuses {
    my ($body, $name) = @_;
    my $accepted = eval { $body->(); 1 };
    ok(!$accepted && length($@), $name);
}
refuses(sub { command($^X, '-e', 'print "valid prefix"; exit 7') }, 'failed command cannot admit partial output');
refuses(sub { bytes("$root/../large") }, 'parent traversal refuses');
refuses(sub { bytes("$root/missing") }, 'missing evidence refuses');
symlink 'large', "$root/link" or die $!;
refuses(sub { bytes("$root/link") }, 'symlink refuses');
refuses(sub { digest("$root/link") }, 'digest retains symlink refusal');
refuses(sub { bytes($root) }, 'directory refuses');
POSIX::mkfifo("$root/pipe", 0600) == 0 or die $!;
refuses(sub { bytes("$root/pipe") }, 'non-regular evidence refuses without blocking');
done_testing();

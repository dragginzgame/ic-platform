# All boundary packages must be registry identities. Runtime fixtures also have
# an older facade through Timers: preserve that admitted graph while requiring
# the exact supplier/generator pair. Standalone proofs retain one facade.
[.packages[] | select(.name == "ic-platform")] as $facades |
[.packages[] | select(.name == "ic-platform-macros")] as $generators |
($facades + $generators) as $packages |
($require_generator | type) == "boolean" and
all($packages[]; .source == "registry+https://github.com/rust-lang/crates.io-index") and
($packages | map([.name, .version]) | unique | length) == ($packages | length) and
([$facades[] | select(.version == $version)] | length) == 1 and
if $require_generator then
    ([$generators[] | select(.version == $version)] | length) == 1
else
    ($facades | length) == 1 and ($generators | length) <= 1 and
    all($generators[]; .version == $version)
end

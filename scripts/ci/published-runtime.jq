[.packages[] | select(.name == "ic-platform" or .name == "ic-platform-macros")] as $packages |
all($packages[]; .source == "registry+https://github.com/rust-lang/crates.io-index") and
any($packages[]; .name == "ic-platform" and .version == $version) and
any($packages[]; .name == "ic-platform-macros" and .version == $version)

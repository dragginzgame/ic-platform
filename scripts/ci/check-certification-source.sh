#!/usr/bin/env bash
set -euo pipefail
# Reconstruct the imported certificate/tree source, including all selected tests.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform/src/certification/upstream}"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/platform-certification-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
while read -r hash admitted file; do
    source="$source_root/$file"
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$admitted" "$source"
    if [[ "$file" == *.rs ]]; then
        perl -0777 -pe '
            s/crate::certification::upstream::/crate::/g;
            s{\Q`HashTreeNode` (internal)\E}{[`HashTreeNode`](hash_tree::HashTreeNode)};
            s{\Q`HashTreeNode::Fork` (internal)\E}{[HashTreeNode::Fork]};
            s{\Q`HashTreeNode::Labeled` (internal)\E}{[HashTreeNode::Labeled]};
            s{\A\Q#![expect(clippy::collapsible_if, reason = "Preserve the upstream tree traversal body.")]\E\n\n}{};
            s{\A\Q#![expect(clippy::useless_borrows_in_formatting, reason = "Preserve original upstream assertions.")]\E\n\n}{};
            s{\Qfeature = "certification" /* Platform: serde selected. */\E}{feature = "serde"}g;
            s{\QAsRef::<[u8]>::as_ref(k).cmp(h.key.as_ref())\E}{k.as_ref().cmp(h.key.as_ref())};
            s~Some\(([nh])\) => \{ // Platform: Rust 2024 implicit borrow\.~Some(ref $1) => {~g;
            s/LookupLabelResult<\x27_, Storage>/LookupLabelResult<Storage>/;
            s/-> LookupResult<\x27_>/-> LookupResult/ if $ARGV =~ m{/hash_tree/tests\.rs$};
            s{\Q// Platform: documentation features belong to the facade.\E}{#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]};
            s{\Q// Platform: unused nested_rb_tree module omitted.\E}{pub mod nested_rb_tree;};
            s{\Q// Platform: unused nested_rb_tree exports omitted.\E}{pub use crate::nested_rb_tree::*;};
            s{\Q// Platform: dead-code admission belongs to the import boundary.\E}{#[allow(dead_code)]};
        ' "$source" > "$scratch/source"
        source="$scratch/source"
    fi
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$hash" "$source"
done <<'HASHES'
6565b40e8cddacbed301f08eeb147db3d3edcd1b711f501badb51f8f76ac2629 624a041e66aab464670ced0b46ded7db1f6add4d750fe0d550e4ad450330e0ca mod.rs
92e49bff681271f709de6417d894dd5cf2add0b9cb3f1595d913947425356188 065952a6fbf3fd654c582fa4c760d7508dbd2e814acd8e11bb9e2e45a7ac477c certificate.rs
309ba5f2003d05445603844a3e02e37048766a1d6acd89189aa48e6059c04fe5 34cd79b7c3c9f28e0dc7e10383b0945773d9c2c0881a370b0ef7e7bee4308673 hash_tree/mod.rs
81f997e6bbdd73369eb4f9f42d0bff104e6b1e016aecd6a4bc8c880a18cb2cd6 7bb17aafadbeec0349e263e44671fc47f478790027fa76b3e8d529144fbfe4e3 hash_tree/tests.rs
ab5d63504393ea749baf9b58e98227d9da8a44739334b7ef39a1e153f02544f0 baf33cc5a13721587c57883899541c0485f9f505a067ac662f4bf7d5bd5701d4 rb_tree/mod.rs
8efed9449a6a14c0292886e17cbb09f973234bebef94b71331b0492029cbcbd0 c5711086e22d977c52b6982468dbac8a519d01406294449627e85bd3956d2b3b rb_tree/tests.rs
0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594 0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594 LICENSE-APACHE
e58df853c8ac56b628c4c8753accb5571324c7e92478e19ca731a31a4a67b21f e58df853c8ac56b628c4c8753accb5571324c7e92478e19ca731a31a4a67b21f Cargo.toml.original
HASHES
echo 'Certification hash/tree/Serde bodies, original tests and Apache license match the import.'

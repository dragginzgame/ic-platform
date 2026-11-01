#!/usr/bin/env bash
set -euo pipefail
# Packaging-only reconstruction. Do not refresh original hashes for crypto edits.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform/src/bls/curve}"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/platform-bls-curve-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
while read -r original admitted file; do
    source="$source_root/$file"
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$admitted" "$source"
    if [[ "$file" == *.rs ]]; then
        perl -0777 -pe '
            s/crate::bls::curve::alloc::/alloc::/g;
            s/crate::bls::curve::/crate::/g;
            s{feature = "bls" /\* Platform: (\w+) selected\. \*/}{feature = "$1"}g;
            s{any\(\) /\* Platform: (\w+) unselected\. \*/}{feature = "$1"}g;
            s{\Q// Platform: std is supplied by the facade.\E}{#![no_std]};
            s{\Q// Platform: documentation features belong to the facade.\E}{#![cfg_attr(docsrs, feature(doc_cfg))]};
            s{\Q// Platform: test macros come from the std prelude.\E}{#[cfg(test)]\n#[macro_use]\nextern crate std;};
            s{// Platform: private documentation attribute: }{}g;
            s{// Platform: unneeded upstream allowance: }{}g;
            s{#(!?)\[expect\(clippy::(too_many_arguments|suspicious_arithmetic_impl), reason = "Preserve upstream arithmetic spelling\."\)\]}{#$1\[allow(clippy::$2)\]}g;
            s{\Q// Platform: preserve original chain; clippy::cognitive_complexity is not selected.\E}{#[allow(clippy::cognitive_complexity)]}g;
            s{\Q// Platform: remove ineffective impl-trait #[must_use].\E}{#[must_use]}g;
            s/r#gen/gen/g;
        ' "$source" > "$scratch/source"
        source="$scratch/source"
    fi
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$original" "$source"
done <<'HASHES'
e72f61bf73fc145c0c33b60fc704478979c108eb52ed42ac30542b537ce0f618 e72f61bf73fc145c0c33b60fc704478979c108eb52ed42ac30542b537ce0f618 COPYRIGHT
ca70f883acfde3d2f07c253b052f0f865dfb2747e9bed44c3c3f2177370c88b9 ca70f883acfde3d2f07c253b052f0f865dfb2747e9bed44c3c3f2177370c88b9 Cargo.toml.original
a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2 a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2 LICENSE-APACHE
23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3 23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3 LICENSE-MIT
dfdef2d9afe913e4b1d657852f0967614e2eb64520d1c62807c2edec67c1cd6d ebd5db2bdcdc9dc7f6fc15db2a0e4595f1df1d14b43c3f201c828e7110d3464c fp.rs
afb56683a7560391facf9a095da0812af3ea46c3e80c6ab50fefdb5e51f5cfcd 8e178c530fa3955cb0b895cc8daa2d29d52093e830547df43af8c13db69c6893 fp12.rs
aa6dd9e1812f4bf5835f229b30e8e50e4b80fb18e00033070d3aeb5c9a0fd8aa 7a3a510d9c06c00563518e6794ed66fbd6b298fcbcbe86d86fbcda331f3114d8 fp2.rs
edb1204f41802926717efc70df12a759753929e71567a5a4c28cfae614213cfd fd0cf626b77b38279aa74a6f75efeec13d4bc2ca729d1a31abe06458f39d61bd fp6.rs
09773d9058ede385df739bcf054e4283fd22421ef5de1888ff0bb2992ba8306a be63cadc6640de6b56f10da4a566a0dbe6b09fc12f66c8542cf92a190178157d g1.rs
48efedd4af43d67250e3537d1ca82a7a5b49e6fc645de289fc78ae8afd02f9ab d42cd4780857cd7117332d4cbe94ee1b04362e14e08ad0bbcd983bbd72e22046 g2.rs
3812cc84c23135016683e2f1144fd93b60aba544a8cb978738dea135f6ab1eb2 14b57e503295a06a3742384085c32c2603c8cc36cfe9398fbc2d0b6edda239f9 hash_to_curve/chain.rs
c743d4132fcd766f26d3cb811689feecedac3ddcc55ff46b347fe7f2cff39919 b865c067a1b3aeff8de9c913022836e745bc4b200c0ae113e76f3d9c1b7350bf hash_to_curve/expand_msg.rs
1da80665f23e35b25cb34aadc757f9d04703e97f621bc880c08c377081f1c69f 736e5bbff1f4de32692295faf595f18f8c4c91bf6a083369bbb1d9d3ac4afb08 hash_to_curve/map_g1.rs
3a6b6896b38f1d102a76cd4abde3032416de0b67ab73c5811d22a5f4e842bde4 fd7b0cc828bb77b39461db9784ea641c5b8ac22abb39a9f698b9a50ae9171a34 hash_to_curve/map_g2.rs
b8f48f4fdafa5695c974218c1a39a0f6359a8c2091823e2742acb964bf01b870 b2fc6f74afaa9e0147fac8139395651bcaf152aa7d98ad845b18408d2fe7ef9a hash_to_curve/map_scalar.rs
27176a83f6b95d40683957b8fbd64c6fd0c79c50d7e1a040c434433bedd44db0 6fa3f3c33807e33c3b99510480f647dd8f48c60ab211df20036191d690c3b72b hash_to_curve/mod.rs
9eebda40d7b08ab3ec1b2dd7e29ebf5448239a34b27302ba408b4a6f1791a612 e19e34bd3973b52423fec6f8eafd8dab6e094c68bf0999e5a9b6e79933331d7d mod.rs
3572e4b93b69a91e240e9528e54bcc994e452086c7bdd1a183f055372435f6b2 3572e4b93b69a91e240e9528e54bcc994e452086c7bdd1a183f055372435f6b2 notes/design.rs
1caa6179831ab45832379212c55fd0c1013c097ec620980b973c2e26488c6319 1caa6179831ab45832379212c55fd0c1013c097ec620980b973c2e26488c6319 notes/serialization.rs
70c6ba720c0566e0fe976cf4db5763a8a4edaf9ad54a14907e502e640f733a7a 81af5e0461a6120cc41ea52821da2e6dba30ad4ffabd49a47fe2e17328ae0bf6 pairings.rs
986f81e7fcb783a668bc4ab55721b22d497c5860cd7136e3cc600d2d9c321475 65e1cca4d1bceb19e5a424ba39707b10ddead4a9587387bd7e0ec08744fad7fd scalar.rs
8acaa1d512b4cf96f4740e4c9afc19ba4618d48fe251c452a19382395bcd0929 8acaa1d512b4cf96f4740e4c9afc19ba4618d48fe251c452a19382395bcd0929 tests/g1_compressed_valid_test_vectors.dat
e25aae8e95c9cd39cdb1eec0100c8245e297510fef0091cbf80b91d49de6b685 e25aae8e95c9cd39cdb1eec0100c8245e297510fef0091cbf80b91d49de6b685 tests/g1_uncompressed_valid_test_vectors.dat
b75bc2e41289adda41e448e56a1ea39b11009a27b02e8676c8b86defedc05deb b75bc2e41289adda41e448e56a1ea39b11009a27b02e8676c8b86defedc05deb tests/g2_compressed_valid_test_vectors.dat
80fd0516c8115b13165589324558649fa94f49e714caa3b735537d073a9cfb65 80fd0516c8115b13165589324558649fa94f49e714caa3b735537d073a9cfb65 tests/g2_uncompressed_valid_test_vectors.dat
44c1013fe46e2c98ad83447b22643bd292d005331602e0882cbaa892b21df378 bbcaf020baac2ed714330d0915069f30d141746bdebf508f7394f2700f3146c5 tests/mod.rs
7a9e3f46f006be867faa0c8cf93031482ba1b7c3f370785620cc1deaa387a041 7a9e3f46f006be867faa0c8cf93031482ba1b7c3f370785620cc1deaa387a041 util.rs
HASHES
echo 'BLS curve arithmetic, constants, original tests/vectors and licenses match ic_bls12_381 0.10.1.'

# Validate the actual package once; messages remain local contract diagnostics.
def require(condition; $message):
  [condition] as $results |
  if ($results | length) > 0 and all($results[]; . == true)
  then . else error($message) end;
. as $metadata |
[.packages[] | select(.name == "ic-platform")] |
if length == 1 then .[0] else error("expected exactly one ic-platform package") end |
require(([.dependencies[] | select((.name == "candid" or .name == "candid_parser") and .kind != "dev")] | length == 0); "Upstream Candid/parser may only be development oracles") |
require(((.features | keys) == $policy[0].features); "Platform feature inventory changed; update the graph matrix and isolation policy") |
require((.features["candid-codec-endpoint"] == ["runtime", "candid-codec", "candid-args", "dep:ic-platform-macros"]); "Owned external endpoints must select the existing runtime and owned codec without upstream Candid") |
require((.features.principal == ["dep:ic-platform-types", "dep:serde"] and (.features | has("principal-external") | not) and .features["principal-self-authenticating"] == ["principal", "ic-platform-types/principal-self-authenticating"] and ([.dependencies[] | select(.name == "ic_principal")] | length == 0)); "Owned Principal and optional self-authentication must not expose registry conversions") |
require(([$metadata.packages[] | select(.name == "ic-platform-types")] | length == 1 and
    .[0].features.default == [] and
    (.[0].features | has("principal-external") | not) and
    .[0].features["principal-self-authenticating"] == ["dep:ic_principal"] and
    ([.[0].dependencies[] | select(.kind == null and
        (.name | test("^(candid(_parser)?|ic-platform(-runtime)?|ic-agent)$")))] | length == 0) and
    ([.[0].dependencies[] | select(.name == "ic_principal" and .kind != "dev")] |
        length == 1 and .[0].optional and .[0].kind == null and .[0].target == null and
        (.[0].uses_default_features | not) and .[0].features == ["self_authenticating"] and
        .[0].req == "^0.1.5" and (.[0].source // "" | startswith("registry+"))) and
    ([.[0].dependencies[] | select(.name == "sha2" and .kind != "dev")] | length == 0));
    "Types must isolate registry Principal behind explicit self-authentication without local cryptography") |
require((.features.default == []); "Platform must declare empty default features") |
require((.features["candid-args"] == [] and .features["raw-rand"] == ["candid-args"]); "Small argument encoding and raw_rand must select no dependency or runtime") |
require((.features["icp-ledger-types"] == ["candid-codec", "principal", "dep:crc32fast", "dep:sha2_11"]); "ICP DTOs use the owned codec and retain only their existing account derivation suppliers") |
require((.features["candid-codec"] == ["principal", "dep:num-bigint", "num-bigint/serde"] and .features["nns-registry-types"] == ["candid-codec", "principal"] and .features["cmc-types"] == ["candid-codec", "principal"] and .features["cycles-ledger-types"] == ["candid-codec", "principal"] and .features["certified-assets-types"] == ["candid-codec"]); "Owned codec and migrated DTOs must not select upstream Candid") |
require((.features["ic-protocol"] == ["principal", "certification-cbor", "dep:ic-transport-types"] and
    .features["ic-protocol-fixtures"] == ["ic-protocol"] and
    ([.dependencies[] | select(.name == "ic-transport-types")] | length == 1 and .[0].optional and .[0].req == "^0.49.1" and .[0].target == "cfg(not(target_family = \"wasm\"))"));
    "IC protocol codecs and separate fixture helpers must retain native transport identity without Agent or runtime") |
require((.links == null and .features.executor == ["ic0", "ic-platform-runtime/executor"] and
    .features.runtime == ["principal", "executor", "ic0", "ic-platform-runtime/runtime"] and
    .features.ic0 == ["dep:ic-platform-runtime", "ic-platform-runtime/ic0"] and
    ([.dependencies[] | select(.name == "slotmap" or .name == "pin-project-lite" or .name == "ic-platform-runtime-guard" or .name == "ic-cdk" or .name == "ic-cdk-executor" or .name == "ic0")] | length == 0)); "Facade must delegate runtime ownership without a second implementation") |
require(([$metadata.packages[] | select(.name == "ic-platform-runtime")] | length == 1 and
    .[0].features.default == [] and .[0].features.executor == ["ic0", "dep:slotmap", "dep:ic-platform-runtime-guard"] and
    .[0].features.ic0 == ["dep:ic0"] and (.[0] as $r |
    $r.links == null and
    $r.features.runtime == ["executor", "dep:ic-platform-types", "dep:pin-project-lite", "dep:serde", "dep:thiserror"] and
    $r.features.cdk == ["runtime", "dep:ic-platform-macros"] and
    ([$r.dependencies[] | select(.kind == null) | .name] | sort) == ["ic-platform-macros", "ic-platform-runtime-guard", "ic-platform-types", "ic0", "pin-project-lite", "serde", "slotmap", "thiserror"] and
    ([$r.dependencies[] | select(.name == "ic0")] | length == 1 and .[0].req == "^1.2.0" and (.[0].uses_default_features | not) and .[0].features == [] and (.[0].source // "" | startswith("registry+"))) and
    all($r.dependencies[] | select(.kind == null); .optional and .target == null) and
    ([$r.dependencies[] | select(.name == "ic-platform-macros")] | .[0].req == ("^" + $r.version)) and
    ([$r.dependencies[] | select(.name == "ic-platform-runtime-guard")] | length == 1 and .[0].optional and .[0].req == ("=" + $r.version)) and
    ([$metadata.packages[] | select(.name == "ic-platform-runtime-guard")] | length == 1 and .[0].version == $r.version and (.[0].dependencies | all(.[]; .kind == "dev")) and .[0].features == {default: []} and .[0].links == "ic-cdk async executor, see https://github.com/dfinity/cdk-rs/blob/links-pin/TROUBLESHOOTING.md") and
    ([$metadata.packages[] | select(.name == "ic-platform-macros")] | length == 1 and .[0].version == $r.version))); "Canonical runtime must retain the exact singleton guard and coordinated generator") |
require((.features["management-canister-types"] == ["principal", "candid-codec", "dep:serde_bytes"] and ([.dependencies[] | select(.name == "ic-management-canister-types")] | length == 1 and .[0].req == "=0.11.0" and .[0].kind == "dev" and (.[0].optional | not) and .[0].target == null)); "Management DTOs must be owned on all targets; registry 0.11.0 is a dev-only oracle") |
require((.features["icrc-ledger-types"] == ["dep:crc32fast", "dep:data-encoding", "principal", "candid-codec", "dep:leb128", "dep:hex", "dep:num-bigint", "dep:num-traits", "dep:serde_bytes", "dep:sha2_11"] and ([.dependencies[] | select(.name == "icrc-ledger-types")] | length == 0)); "Selected ledger DTOs must not acquire the retired upstream package") |
require((.features.certification == ["dep:ic-certification"] and
    .features["certification-cbor"] == ["certification", "dep:serde", "dep:serde_bytes", "dep:serde_cbor", "dep:thiserror"] and
    .features["canister-sig-creation"] == ["principal", "certification", "dep:sha2_11"] and
    .features.agent == ["principal", "dep:ic-agent"] and
    ([.dependencies[] | select(.name == "ic-certification")] | length == 1 and .[0].optional and .[0].target == null and (.[0].uses_default_features | not) and .[0].features == ["serde"]) and
    ([.dependencies[] | select(.name == "ic-canister-sig-creation")] | length == 1 and .[0].kind == "dev" and (.[0].optional | not) and .[0].req == "^1.3.1" and .[0].target == null and (.[0].uses_default_features | not)) and
    ([.dependencies[] | select(.name == "ic-signature-verification")] | length == 1 and .[0].kind == "dev" and (.[0].optional | not) and .[0].req == "^0.3.0" and .[0].target == null and (.[0].uses_default_features | not)));
    "Certification must use registry identities; signature protocol modules use owned issuers and development-only oracles") |
require((.features.bls == ["dep:ic-verify-bls-signature", "dep:ic_bls12_381"] and
    .features["signature-verification"] == ["bls", "canister-sig-creation", "dep:serde_bytes", "dep:serde_cbor", "dep:thiserror"] and
    ([.dependencies[] | select(.name == "ic-verify-bls-signature")] | length == 1 and .[0].optional and .[0].req == "^0.6.0" and (.[0].uses_default_features | not) and .[0].features == ["alloc"] and .[0].target == null) and
    ([.dependencies[] | select(.name == "ic_bls12_381")] | length == 1 and .[0].optional and .[0].req == "^0.10.1" and (.[0].uses_default_features | not) and .[0].features == ["groups"] and .[0].target == null) and
    ([.dependencies[] | select(.name == "digest" or .name == "ff" or .name == "group" or .name == "pairing" or .name == "rand_core" or .name == "subtle" or .name == "rand" or .name == "rand_xorshift" or .name == "sha3")] | length == 0)); "BLS must delegate cryptography upstream with alloc, identity admission and no RNG capability") |
require((.features.ed25519 == ["dep:curve25519-dalek", "dep:ed25519-dalek"] and ([.dependencies[] | select(.name == "curve25519-dalek" or .name == "ed25519-dalek")] | length == 2 and all(.[]; .optional and (.uses_default_features | not) and .features == [] and .target == null))); "Ed25519 must remain optional on native/Wasm without upstream default/RNG features") |
require((.features.ecdsa == ["dep:crypto-bigint", "dep:k256"] and ([.dependencies[] | select(.name == "k256")] | length == 1 and .[0].req == "^0.14.0" and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["ecdsa"] and .[0].target == null) and
    ([.dependencies[] | select(.name == "crypto-bigint")] | length == 1 and .[0].req == "^0.7.10" and .[0].optional and (.[0].uses_default_features | not) and .[0].target == null)); "ECDSA must remain optional without default/RNG features and enforce its published bigint security floor") |
true

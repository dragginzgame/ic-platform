# Validate the actual package once; messages remain local contract diagnostics.
def require(condition; $message):
  [condition] as $results |
  if ($results | length) > 0 and all($results[]; . == true)
  then . else error($message) end;
. as $metadata |
[.packages[] | select(.name == "ic-platform")] |
if length == 1 then .[0] else error("expected exactly one ic-platform package") end |
require((.features.candid == ["principal-external", "dep:candid", "candid/bignum"] and
    .features["candid-service"] == ["candid", "candid/printer"]); "Candid service generation must remain explicitly selected") |
require(((.features | keys) == $policy[0].features); "Platform feature inventory changed; update the graph matrix and isolation policy") |
require((.features["candid-endpoint"] == ["cdk", "candid"]); "External Candid endpoints must explicitly select the existing generator and Candid codec") |
require((.features.principal == ["dep:ic-platform-types", "dep:serde"] and .features["principal-external"] == ["principal", "ic-platform-types/principal-external"] and .features["principal-self-authenticating"] == ["principal", "ic-platform-types/principal-self-authenticating"] and ([.dependencies[] | select(.name == "ic_principal")] | length == 0)); "Owned Principal, external byte conversion and self-authentication must remain separate") |
require(([$metadata.packages[] | select(.name == "ic-platform-types")] | length == 1 and
    .[0].features.default == [] and
    .[0].features["principal-external"] == ["dep:ic_principal", "ic_principal/serde"] and
    .[0].features["principal-self-authenticating"] == ["dep:sha2_11"] and
    ([.[0].dependencies[] | select(.kind == null and
        (.name | test("^(candid(_parser)?|ic-platform(-runtime)?|ic-agent)$")))] | length == 0) and
    ([.[0].dependencies[] | select(.name == "ic_principal" and .kind == null)] |
        length == 1 and .[0].optional and .[0].target == null and (.[0].uses_default_features | not) and .[0].features == ["convert"]));
    "Types external Principal must supply upstream Serde without default hashing or a codec") |
require((.features.default == []); "Platform must declare empty default features") |
require((.links == null and .features.executor == ["ic0", "ic-platform-runtime/executor"] and
    .features.runtime == ["principal", "executor", "ic0", "ic-platform-runtime/runtime"] and
    .features.cdk == ["runtime", "dep:ic-platform-macros"] and
    .features.ic0 == ["dep:ic-platform-runtime", "ic-platform-runtime/ic0"] and
    ([.dependencies[] | select(.name == "slotmap" or .name == "pin-project-lite" or .name == "ic-platform-runtime-guard" or .name == "ic-cdk" or .name == "ic-cdk-executor" or .name == "ic0")] | length == 0)); "Facade must delegate runtime ownership without a second implementation") |
require(([$metadata.packages[] | select(.name == "ic-platform-runtime")] | length == 1 and
    .[0].features.default == [] and .[0].features.executor == ["ic0", "dep:slotmap", "dep:ic-platform-runtime-guard"] and
    .[0].features.ic0 == [] and (.[0] as $r |
    $r.links == null and
    $r.features.runtime == ["executor", "dep:ic-platform-types", "dep:pin-project-lite", "dep:serde", "dep:thiserror"] and
    $r.features.cdk == ["runtime", "dep:ic-platform-macros"] and
    ([$r.dependencies[] | select(.kind == null) | .name] | sort) == ["ic-platform-macros", "ic-platform-runtime-guard", "ic-platform-types", "pin-project-lite", "serde", "slotmap", "thiserror"] and
    all($r.dependencies[] | select(.kind == null); .optional and .target == null) and
    ([$r.dependencies[] | select(.name == "ic-platform-macros")] | .[0].req == ("^" + $r.version)) and
    ([$r.dependencies[] | select(.name == "ic-platform-runtime-guard")] | length == 1 and .[0].optional and .[0].req == ("=" + $r.version)) and
    ([$metadata.packages[] | select(.name == "ic-platform-runtime-guard")] | length == 1 and .[0].version == $r.version and .[0].dependencies == [] and .[0].features == {default: []} and .[0].links == "ic-cdk async executor, see https://github.com/dfinity/cdk-rs/blob/links-pin/TROUBLESHOOTING.md") and
    ([$metadata.packages[] | select(.name == "ic-platform-macros")] | length == 1 and .[0].version == $r.version))); "Canonical runtime must retain the exact singleton guard and coordinated generator") |
require((.features.secp256k1 == ["principal-external", "dep:ic-secp256k1"] and ([.dependencies[] | select(.name == "ic-secp256k1")] | length == 1 and .[0].optional and .[0].req == "=0.3.0" and .[0].target == "cfg(not(target_family = \"wasm\"))")); "secp256k1 must remain exact, optional and native-only") |
require((.features["management-canister-types"] == ["principal-external", "dep:candid", "candid/bignum", "candid/serde_bytes", "dep:serde_bytes", "dep:ic-management-canister-types"] and ([.dependencies[] | select(.name == "ic-management-canister-types")] | length == 1 and .[0].req == "=0.11.0" and .[0].optional and .[0].target == "cfg(not(target_family = \"wasm\"))")); "Platform must own selected Wasm management DTOs and retain native-only upstream 0.11.0") |
require((.features["icrc-ledger-types"] == ["dep:crc32fast", "dep:data-encoding", "principal-external", "dep:candid", "candid/bignum", "candid/serde_bytes", "dep:hex", "dep:num-bigint", "dep:num-traits", "dep:serde_bytes", "dep:sha2"] and ([.dependencies[] | select(.name == "icrc-ledger-types")] | length == 0)); "Selected ledger DTOs must not acquire the retired upstream package") |
require((.features.certification == ["dep:ic-certification"] and
    .features["canister-sig-creation"] == ["principal-external", "certification", "dep:ic-canister-sig-creation"] and
    .features.agent == ["principal-external", "dep:ic-agent"] and
    ([.dependencies[] | select(.name == "ic-certification")] | length == 1 and .[0].optional and .[0].target == null and (.[0].uses_default_features | not) and .[0].features == ["serde"]) and
    ([.dependencies[] | select(.name == "ic-canister-sig-creation")] | length == 1 and .[0].optional and .[0].req == "^1.3.1" and .[0].target == null and (.[0].uses_default_features | not)) and
    ([.dependencies[] | select(.name == "ic-signature-verification")] | length == 1 and .[0].optional and .[0].req == "^0.3.0" and .[0].target == null and (.[0].uses_default_features | not)));
    "Certification must use registry identities; signature key adapters preserve owned issuers") |
require((.features.bls == ["dep:ic-verify-bls-signature", "dep:ic_bls12_381"] and
    .features["signature-verification"] == ["bls", "canister-sig-creation", "dep:serde_bytes", "dep:serde_cbor", "dep:ic-signature-verification"] and
    ([.dependencies[] | select(.name == "ic-verify-bls-signature")] | length == 1 and .[0].optional and .[0].req == "^0.6.0" and (.[0].uses_default_features | not) and .[0].features == ["alloc"] and .[0].target == null) and
    ([.dependencies[] | select(.name == "ic_bls12_381")] | length == 1 and .[0].optional and .[0].req == "^0.10.1" and (.[0].uses_default_features | not) and .[0].features == ["groups"] and .[0].target == null) and
    ([.dependencies[] | select(.name == "digest" or .name == "ff" or .name == "group" or .name == "pairing" or .name == "rand_core" or .name == "subtle" or .name == "rand" or .name == "rand_xorshift" or .name == "sha3")] | length == 0)); "BLS must delegate cryptography upstream with alloc, identity admission and no RNG capability") |
require((.features.ed25519 == ["dep:curve25519-dalek", "dep:ed25519-dalek"] and ([.dependencies[] | select(.name == "curve25519-dalek" or .name == "ed25519-dalek")] | length == 2 and all(.[]; .optional and (.uses_default_features | not) and .features == [] and .target == null))); "Ed25519 must remain optional on native/Wasm without upstream default/RNG features") |
require((.features.ecdsa == ["dep:crypto-bigint", "dep:k256"] and ([.dependencies[] | select(.name == "k256")] | length == 1 and .[0].req == "^0.14.0" and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["ecdsa"] and .[0].target == null) and
    ([.dependencies[] | select(.name == "crypto-bigint")] | length == 1 and .[0].req == "^0.7.10" and .[0].optional and (.[0].uses_default_features | not) and .[0].target == null)); "ECDSA must remain optional without default/RNG features and enforce its published bigint security floor") |
true

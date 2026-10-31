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
require((.features.principal == ["dep:ic-platform-types", "dep:serde", "dep:thiserror"] and .features["principal-external"] == ["principal", "ic-platform-types/principal-external", "dep:ic_principal"] and .features["principal-self-authenticating"] == ["principal-external", "ic-platform-types/principal-self-authenticating"] and ([.dependencies[] | select(.name == "ic_principal")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["convert"])); "Owned Principal, external byte conversion and self-authentication must remain separate") |
require((.features.default == []); "Platform must declare empty default features") |
require((.features.executor == ["ic0", "dep:slotmap", "dep:ic-platform-runtime-guard"] and ([.dependencies[] | select(.name == "ic-cdk-executor" or .name == "ic0" or .name == "smallvec")] | length == 0) and .links == null); "Platform must own the sole executor, use Vec task lists and retain its Cargo singleton guard") |
require((. as $p | ([.dependencies[] | select(.name == "ic-platform-runtime-guard")] |
    length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].target == null and .[0].req == ("=" + $p.version)) and
    ([$metadata.packages[] | select(.name == "ic-platform-runtime-guard")] |
    length == 1 and .[0].version == $p.version and .[0].dependencies == [] and .[0].features == {default: []} and
    .[0].links == "ic-cdk async executor, see https://github.com/dfinity/cdk-rs/blob/links-pin/TROUBLESHOOTING.md")); "Executor must select the exact coordinated, dependency-free singleton guard") |
require((. as $p | .features.cdk == ["runtime", "dep:ic-platform-macros"] and .features.runtime == ["principal", "executor", "ic0", "dep:pin-project-lite", "dep:thiserror"] and ([.dependencies[] | select(.name == "ic-cdk")] | length == 0) and ([.dependencies[] | select(.name == "ic-platform-macros")] | length == 1 and .[0].optional and .[0].req == ("^" + $p.version))); "CDK must select only the owned adapters, qualified executor/ic0 and coordinated generator") |
require((.features.secp256k1 == ["principal-external", "dep:ic-secp256k1"] and ([.dependencies[] | select(.name == "ic-secp256k1")] | length == 1 and .[0].optional and .[0].req == "=0.3.0" and .[0].target == "cfg(not(target_family = \"wasm\"))")); "secp256k1 must remain exact, optional and native-only") |
require((.features.ic0 == []); "IC0 must use owned bindings without package dependencies") |
require((.features["management-canister-types"] == ["principal-external", "dep:candid", "candid/bignum", "candid/serde_bytes", "dep:serde_bytes", "dep:ic-management-canister-types"] and ([.dependencies[] | select(.name == "ic-management-canister-types")] | length == 1 and .[0].req == "=0.11.0" and .[0].optional and .[0].target == "cfg(not(target_family = \"wasm\"))")); "Platform must own selected Wasm management DTOs and retain native-only upstream 0.11.0") |
require((.features["icrc-ledger-types"] == ["dep:crc32fast", "dep:data-encoding", "principal-external", "dep:candid", "candid/bignum", "candid/serde_bytes", "dep:hex", "dep:num-bigint", "dep:num-traits", "dep:serde_bytes", "dep:sha2"] and ([.dependencies[] | select(.name == "icrc-ledger-types")] | length == 0)); "Selected ledger DTOs must not acquire the retired upstream package") |
require((.features.bls == ["dep:ic-verify-bls-signature"] and ([.dependencies[] | select(.name == "ic-verify-bls-signature")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["alloc"])); "BLS must be optional with alloc and no upstream default/RNG features") |
require((.features.ed25519 == ["dep:curve25519-dalek", "dep:ed25519-dalek"] and ([.dependencies[] | select(.name == "curve25519-dalek" or .name == "ed25519-dalek")] | length == 2 and all(.[]; .optional and (.uses_default_features | not) and .features == [] and .target == null))); "Ed25519 must remain optional on native/Wasm without upstream default/RNG features") |
require((.features["stable-structures"] == ["dep:ic-stable-structures"] and ([.dependencies[] | select(.name == "ic-stable-structures")] | length == 1 and .[0].optional and .[0].req == "=0.7.2" and (.[0].uses_default_features | not) and .[0].features == [] and .[0].target == null)); "Storage substrate must remain optional, portable and aligned with Memory =0.7.2") |
require((.features.ecdsa == ["dep:k256"] and ([.dependencies[] | select(.name == "k256")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["ecdsa"] and .[0].target == null)); "ECDSA must remain optional on native/Wasm without default/RNG features") |
true

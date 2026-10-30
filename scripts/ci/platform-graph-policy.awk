# Shared package classifications. Each caller still observes and admits its own graph.
function platform_host_capability(name) {
    if (name == "candid_parser") return "candid-parser"
    if (name == "ic-agent") return "agent"
    if (name == "pocket-ic") return "pocket-ic"
    if (name == "ic-secp256k1") return "secp256k1"
    if (name == "ic-testkit" || name == "ic-testkit-candid") return "testkit"
    if (name == "ic-management-canister-types") return "management-native"
    return ""
}
function platform_forbidden(family, name) {
    if (family == "minimal")
        return name ~ /^(ic_principal|serde|serde_core|serde_derive|sha2|digest|crc32fast|data-encoding|candid|ic-agent)$/
    if (family == "identity")
        return name ~ /^(icrc-ledger-types|icrc-cbor|minicbor|base32|candid_parser|ic-agent|pocket-ic)$/
    if (family == "runtime-host")
        return platform_host_capability(name) != "" && !(allow_native_management && name == "ic-management-canister-types")
    if (family == "retired-cdk")
        return name ~ /^ic-cdk/ || name == "ic0"
    if (family == "external-codecs")
        return name ~ /^(ic-cdk[^ ]*|ic0|ic-platform-macros|ic-agent|pocket-ic|ic-testkit|candid_parser|reqwest|tokio)$/
    return 0
}
# Standalone use reports whether the graph contains a forbidden package.
family != "" { if (platform_forbidden(family, $1)) forbidden_package = 1 }
END { if (family != "") exit !forbidden_package }

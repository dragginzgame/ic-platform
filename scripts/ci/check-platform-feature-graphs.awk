# Validate every selection against its cached graph in one process per target.
# Input TSV: original selection, target-aware cache key. Graphs remain Cargo output.
BEGIN { FS = "\t" }
function enabled(name) { return index("," selection ",", "," name ",") != 0 }
function reject(reason) {
    print target ": " selection ": " reason " (" graph ")" > "/dev/stderr"
    exit 1
}
{
    if (NF != 2 || $1 == "" || $2 == "") reject("invalid selection mapping")
    selection = $1
    key = $2
    graph = cache "/" target "-" key ".txt"
    if (!(key in loaded)) {
        while ((status = (getline line < graph)) > 0) {
            # Only retain policy-relevant lines; ordinary transitive nodes cannot
            # affect admission. Keep complete lines for source/identity checks.
            split(line, fields, " ")
            if (fields[1] ~ /^(ic|candid|pocket-ic|ed25519-dalek|curve25519-dalek|k256|blake3)/ ||
                fields[1] ~ /^(rand|rand_chacha|getrandom|pretty|serde|serde_core|serde_derive|sha2|digest|crc32fast|data-encoding|serde_cbor|half)$/)
                lines[key, ++count[key]] = line
            nonempty = 1
        }
        close(graph)
        if (status < 0 || !nonempty) reject("missing or empty graph")
        nonempty = 0
        loaded[key] = 1
    }
    native = target !~ /^wasm/
    parser = native && enabled("candid-parser")
    wire_schema = native && enabled("wire-schema")
    agent = native && enabled("agent")
    pocket = native && enabled("pocket-ic")
    secp = native && enabled("secp256k1")
    gzip = native && enabled("gzip")
    external_endpoint = enabled("candid-endpoint")
    cdk = external_endpoint || enabled("cdk")
    runtime = cdk || enabled("runtime")
    executor = runtime || enabled("executor")
    verification = enabled("signature-verification")
    creation = verification || enabled("canister-sig-creation")
    bls = verification || agent || enabled("bls")
    raw_system = executor || enabled("ic0")
    certification = pocket || creation || agent || enabled("certification")
    management = enabled("management-canister-types")
    ledger = enabled("icrc-ledger-types")
    external_dto = enabled("cmc-types") || enabled("icp-ledger-types") || enabled("cycles-ledger-types") || enabled("nns-registry-types") || enabled("certified-assets-types")
    candid = external_endpoint || pocket || enabled("candid") || enabled("candid-service") || parser || agent || management || ledger || external_dto
    principal = enabled("wire-schema") || enabled("principal") || cdk || enabled("runtime") || creation || enabled("candid") || enabled("candid-service") || management || ledger || external_dto || enabled("agent") || enabled("candid-parser") || enabled("pocket-ic") || enabled("secp256k1")
    minimal = !principal && !gzip && !enabled("bls") && !enabled("ecdsa") && !enabled("ed25519") && !enabled("certification") && !enabled("stable-structures")
    found_storage = 0
    found_ed = 0
    found_curve = 0
    found_secp = 0
    found_k256 = 0
    found_pocket = 0
    found_blake3 = 0
    found_gzip = 0
    for (i = 1; i <= count[key]; i++) {
        line = lines[key, i]
        n = split(line, f, " ")
        name = f[1]; version = f[2]
        if (minimal && name ~ /^(ic_principal|serde|serde_core|serde_derive|sha2|digest|crc32fast|data-encoding)$/)
            reject("application types in minimal executor/IC0 graph")
        if (name == "ic-representation-independent-hash" && !(agent || pocket))
            reject("reference-only signature hashing in production")
        if ((name == "serde_cbor" || name == "half") && !(verification || agent || pocket || parser || ledger))
            reject("unselected CBOR parser")
        if (name == "pretty" && !(enabled("candid-service") || parser || agent || pocket || management || ledger))
            reject("unselected Candid service printer")
        registry = n == 2 || (n == 3 && f[3] == "(*)")
        if (name == "ic-host-artifacts") {
            qualified_gzip = gzip_source == "" ? registry : \
                (f[3] == gzip_source && (n == 3 || (n == 4 && f[4] == "(*)")))
            if (!gzip || !qualified_gzip) reject("unselected or unqualified host gzip codec")
            found_gzip = 1
        }
        if (name == "blake3") {
            if (!enabled("blake3") || !registry) reject("unselected or non-registry BLAKE3")
            found_blake3 = 1
        }
        # rand_core supplies traits and is permitted. Entropy/RNG implementations
        # belong only to the selected native host capabilities that require them.
        if (name ~ /^(rand|rand_chacha|getrandom)$/ && !(agent || pocket || secp || parser))
            reject("unselected RNG/entropy scaffolding")
        if (name == "k256") {
            if (!(enabled("ecdsa") || agent || secp) || !registry) reject("unselected K256")
            found_k256 = 1
        }
        if (name == "ed25519-dalek" || name == "curve25519-dalek") {
            if (!(enabled("ed25519") || agent) || !registry) reject("unselected Dalek")
            if (name == "ed25519-dalek") found_ed = 1
            else found_curve = 1
        }
        if (name == "ic-secp256k1") {
            if (!secp || version != "v0.3.0" || !registry) reject("unqualified secp256k1")
            found_secp = 1
        }
        if (name == "pocket-ic") {
            if (!pocket || version != "v16.1.0" || !registry) reject("unqualified PocketIC")
            found_pocket = 1
        }
        if (name == "ic-testkit" || name == "ic-testkit-candid") reject("Testkit in production")
        if (name ~ /^(ic-wire|ic-wire-derive)$/ && !wire_schema) reject("unselected host Wire metadata")
        if (name == "candid_parser" && !parser) reject("unselected host parser")
        if (name == "ic-agent" && !agent) reject("unselected host Agent")
        if (name ~ /^ic-cdk/) {
            if (name == "ic-cdk-executor") {
                if (!executor || version != "v2.0.0") reject("unqualified executor")
            } else reject("full CDK runtime/generator is retired")
            if (!registry) reject("non-registry executor")
        }
        if (name == "ic0" && (!raw_system || version != "v1.2.0" || !registry)) reject("unqualified ic0")
        if (name == "ic-platform-macros" && (!cdk || (line != generator && line != generator " (*)"))) reject("unqualified generator")
        if (name == "ic-canister-sig-creation" || name == "ic-signature-verification") reject("retired signature package")
        if (name == "ic-verify-bls-signature" && (!bls || version != "v0.6.0" || !registry)) reject("unqualified BLS")
        if (name == "ic_bls12_381" && (!bls || version != "v0.10.1" || !registry)) reject("unqualified BLS curve")
        if (name == "ic-certification" && (!certification || version != "v3.2.0" || !registry)) reject("unqualified certification")
        if (name == "ic-management-canister-types" && !(registry && ((management && version == "v0.11.0") || (pocket && version == "v0.8.0")))) reject("unselected management types")
        if (name == "icrc-ledger-types" && !ledger) reject("unselected ledger types")
        if (name == "ic-error-types" && !(agent || pocket)) reject("runtime error catalogue")
        if (name == "ic-stable-structures") {
            if (!enabled("stable-structures") || version != "v0.7.2" || !registry) reject("unqualified storage substrate")
            found_storage = 1
        }
        if ((name == "candid" || name == "candid_derive") && !candid) reject("unexpected Candid")
    }
    if (enabled("stable-structures") && !found_storage) reject("missing selected storage substrate")
    if (enabled("ed25519") && (!found_ed || !found_curve)) reject("missing selected Dalek")
    if (pocket && !found_pocket) reject("missing selected PocketIC")
    if (secp && !found_secp) reject("missing selected secp256k1")
    if (enabled("ecdsa") && !found_k256) reject("missing selected K256")
    if (enabled("blake3") && !found_blake3) reject("missing selected BLAKE3")
    if (gzip && !found_gzip) reject("missing selected host gzip codec")
}

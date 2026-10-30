// Assemble an isolated codec using registry Candid's public types and traits.
const fs = require('fs');
const [source, output] = process.argv.slice(2);
const read = p => fs.readFileSync(`${source}/src/${p}`, 'utf8');
const write = (p, text) => {
  fs.mkdirSync(`${output}/${p.substring(0, p.lastIndexOf('/'))}`, { recursive: true });
  fs.writeFileSync(`${output}/${p}`, text.replace(/((?:TypeInner|ConsType)::\w+\()ref /g, '$1').replace('    pub(crate) fn overflows', '    #[expect(dead_code, reason = "Preserved upstream recursion helper; only subtype caching reads this counter")]\n    pub(crate) fn overflows'));
};
for (const name of ['de', 'ser', 'binary_parser']) write(`src/${name}/mod.rs`, read(`${name}.rs`));
let de = read('de.rs');
de = de.replace('\n', '\nuse crate::compat::TypeEnvExt;\n');
write('src/de/mod.rs', de);
const internal = read('types/internal.rs');
const diagnostics = internal.slice(internal.indexOf('/// Budget for a type named'), internal.indexOf('#[allow(clippy::result_unit_err)]'));
const utils = read('utils.rs');
const recursion = utils.slice(utils.indexOf('/// Guard against stack overflow'), utils.indexOf('pub fn check_unique'));
const env = read('types/type_env.rs');
const method = env.slice(env.indexOf('    pub(crate) fn trace_type_with_depth'), env.indexOf('    pub fn as_func')).replace('pub(crate) fn', 'fn');
write('src/compat/mod.rs', `use candid::{Error, Result};\nuse candid::types::{Type, TypeEnv, TypeInner};\nuse candid::types::internal::text_size;\nuse std::fmt;\n${diagnostics}\n${recursion}\npub(crate) trait TypeEnvExt {\n    fn trace_type_with_depth<'a>(&'a self, t: &'a Type, depth: &RecursionDepth) -> Result<Type>;\n}\nimpl TypeEnvExt for TypeEnv {\n${method}}\n`);
// Subtyping retains registry Candid's independent recursion tracker. The copied
// decoder only needs guard(); keep the unused helper for source comparison.
write('src/lib.rs', `//! Packaging experiment, not a supported codec.\npub use candid::{CandidType, Error, Int, Nat, Reserved, Result};\npub use candid::error;\npub mod de;\npub mod ser;\npub mod binary_parser;\nmod compat;\nmod utils { pub(crate) use crate::compat::RecursionDepth; }\npub mod types {\n    pub use candid::types::*;\n    pub mod internal {\n        pub use candid::types::internal::*;\n        pub(crate) use crate::compat::{elide_large, MAX_DIAGNOSTIC_LIST_LEN, MAX_DIAGNOSTIC_TYPE_LEN};\n    }\n}\nimpl From<binary_parser::ReadError> for Error {\n    fn from(error: binary_parser::ReadError) -> Self { Self::msg(error) }\n}\n`);
fs.copyFileSync(`${source}/LICENSE`, `${output}/LICENSE`);
// Differential runner retains the existing registry-derived contract types.
const here = __dirname;
const original = fs.readFileSync(`${here}/../candid-reader/harness/src/bin/differential/main.rs`, 'utf8');
let runner = original.replace('    de::{DecoderConfig, IDLDeserialize},\n', '');
runner = `#[cfg(not(feature = "owned-codec"))]\nuse candid as selected;\n#[cfg(feature = "owned-codec")]\nuse codec as selected;\nuse selected::de::{DecoderConfig, IDLDeserialize};\n` + runner;
runner = runner.replace('#[cfg(feature = "baseline-reader")]', '#[cfg(all(feature = "baseline-reader", not(feature = "owned-codec")))]');
runner = runner.replace('../../tests/fixtures/', '../../fixtures/');
runner = runner.replaceAll('candid::binary_parser::Header', 'selected::binary_parser::Header');
runner = runner.replace('hex(&candid::encode_one(v)?)', 'hex(&encode(v)?)');
runner += `\nfn encode<T: CandidType>(value: T) -> candid::Result<Vec<u8>> {\n    let mut builder = selected::ser::IDLBuilder::new();\n    builder.arg(&value)?;\n    builder.serialize_to_vec()\n}\n`;
fs.mkdirSync(`${output}/../consumer/src/bin/differential`, {recursive:true});
fs.writeFileSync(`${output}/../consumer/src/bin/differential/main.rs`, runner);

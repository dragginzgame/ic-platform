// Pinned consumer adaptation in ignored storage; never edit the Auth checkout.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve,relative} from 'node:path';
import {adoptTooling} from '../tooling-selection.mjs';
const root=process.cwd();const source=resolve(process.env.AUTH_SOURCE??'../ic-auth');
const revision='b78991e9833320692921bf766e51ad361c9c8c4d';
const output=resolve(`target/auth-runtime/${Date.now()}`);mkdirSync(output,{recursive:true});
const run=(cmd,args,cwd=root)=>{const r=spawnSync(cmd,args,{cwd,encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(r.status,0,r.stderr);return r.stdout;};
run('git',['archive',`--output=${output}/auth.tar`,revision,'Cargo.toml','Cargo.lock','LICENSE.md','crates','qualification'],source);
run('tar',['-xf',`${output}/auth.tar`,'-C',output]);
run('git',['apply',`--directory=${relative(root,output)}`,`${root}/qualification/auth-runtime/testkit.patch`],root);
run('git',['apply',`--directory=${relative(root,output)}`,`${root}/qualification/auth-crypto/consumer.patch`],root);
const edit=(path,fn)=>{const file=`${output}/${path}`;writeFileSync(file,fn(readFileSync(file,'utf8')));};
edit('Cargo.toml',s=>s.replace('ic-platform-types = { version = "0.20.4",',`ic-platform-types = { path = ${JSON.stringify(`${root}/crates/ic-platform-types`)}, version = "=0.21.2",`));
for (const name of ['Cargo.toml','qualification/Cargo.toml']) edit(name,s=>s
  .replace('ic-platform = { version = "0.33.7",', `ic-platform = { path = ${JSON.stringify(`${root}/crates/ic-platform`)}, version = "=0.34.14",`)
  .replace('ic-platform-types = { version = "0.20.4",', `ic-platform-types = { path = ${JSON.stringify(`${root}/crates/ic-platform-types`)}, version = "=0.21.2",`)
  .replace('ic-wire = "0.24.1"', 'ic-wire = "=0.24.2"'));
edit('crates/ic-auth/Cargo.toml',s=>s.replace('ic-platform/signature-verification','ic-platform/canister-signature-authentication'));
for (const name of ['ic-auth-certified-conformance','ic-auth-host-fixture','ic-auth-host-conformance'])
  edit(`qualification/${name}/Cargo.toml`,s=>s
    .replace('["certification-cbor", "ic-protocol-fixtures"]','["ecdsa", "certificate-construction", "certificate-fixture-signing", "canister-signature-authentication"]')
    .replace('["canister-sig-creation", "certification-cbor", "ed25519"]','["certificate-construction", "ed25519"]')
    .replace('["certification-cbor", "canister-sig-creation"]','["certificate-construction", "canister-signature-authentication"]'));
edit('qualification/Cargo.toml',s=>s.replace('ic-platform-runtime = { version = "0.22.5",',`ic-platform-runtime = { path = ${JSON.stringify(`${root}/crates/ic-platform-runtime`)}, version = "=0.23.17",`).replace('unsafe_code = "forbid"','unsafe_code = "deny"'));
edit('qualification/ic-auth-host-fixture/Cargo.toml',s=>s.replace('features = ["cdk"]','features = ["entrypoint-macros"]'));
edit('qualification/Cargo.toml',s=>s.replace('ic-testkit = "0.106.0"', 'ic-testkit = "0.109.0"'));
edit('qualification/ic-auth-host-conformance/Cargo.toml', s => s.replace('[dev-dependencies]', '[dev-dependencies]\nic-management-canister-types = "=0.8.0"'));
for(const file of ['canister.rs','receiver/canister.rs','browser/canister.rs','enrollment/canister.rs'])edit(`qualification/ic-auth-host-fixture/src/${file}`,s=>{
  s=s.replace('use ic_platform_runtime::{cdk, cdk::api, ic0};','use ic_platform_runtime::{self as cdk, system as api};');
  s=s.replaceAll('api::msg_caller()', 'api::caller()').replaceAll('ic0::msg_arg_data_size()', 'api::argument_size()').replaceAll('api::certified_data_set(', 'api::set_certified_data(&').replaceAll('api::data_certificate()', 'api::data_certificate(RESPONSE.bytes).unwrap()');
  let adapters='\nfn __runtime_fail<E: std::fmt::Debug>(error: E) -> ! { api::trap(&format!("endpoint refused: {error:?}")) }\n';
  s=s.replace(/#\[cdk::(update|query)\(([\s\S]*?)\)\]\n(async )?fn (\w+)\(([^)]*)\)([^\{]*)\{/g,(whole,kind,opts,asyncness,name,parameters,returns)=>{
    const get=k=>opts.match(new RegExp(`${k} = "([^"]+)"`))?.[1];
    const guard=get('guard'),decoder=get('decode_with'),encoder=get('encode_with'),method=get('name');assert(guard&&decoder&&encoder&&method);
    const input=parameters.trim()?parameters.slice(parameters.indexOf(':')+1).trim():'()';
    const result=returns.trim().startsWith('->')?returns.trim().slice(2).trim():'()';
    const encoderNoArgs=new RegExp(`fn ${encoder}\\(\\)`).test(s);
    adapters+=`\nfn __${name}_decode(bytes: &[u8]) -> Result<${input}, String> { Ok(${decoder}(bytes.to_vec())) }\nfn __${name}_encode(value: ${result}) -> Result<Vec<u8>, String> { Ok(${encoder}(${encoderNoArgs?'':'value'})) }\n`;
    if(encoderNoArgs)adapters=adapters.replace(`fn __${name}_encode(value: ()`, `fn __${name}_encode((): ()`);
    return `#[cdk::${kind}(runtime = ic_platform_runtime, name = "${method}", guard = ${guard}, decode = __${name}_decode, encode = __${name}_encode, max_bytes = 1048576, on_error = __runtime_fail)]\n${asyncness??''}fn ${name}(${parameters.trim()?parameters:'(): ()'})${returns}{`;
  });
  if(file==='canister.rs'){
    s=s.replace('#[cdk::init(decode_with = "decode_install")]', '#[cdk::init(runtime = ic_platform_runtime, guard = __runtime_admit, decode = __install_decode, max_bytes = 1048576, on_error = __runtime_fail)]');
    s=s.replace('#[cdk::post_upgrade]\nfn lost_volatile_state()', '#[cdk::post_upgrade(runtime = ic_platform_runtime, guard = __runtime_admit, decode = __empty_decode, max_bytes = 0, on_error = __runtime_fail)]\nfn lost_volatile_state((): ())');
    adapters+='\nfn __runtime_admit() -> Result<(), String> { Ok(()) }\nfn __install_decode(bytes: &[u8]) -> Result<ic_auth_protocol_types::Principal, String> { Ok(decode_install(bytes.to_vec())) }\nfn __empty_decode(bytes: &[u8]) -> Result<(), String> { if bytes.is_empty() { Ok(()) } else { Err("unexpected upgrade input".into()) } }\n';
  }
  return s+adapters;
});
edit('qualification/ic-auth-host-fixture/src/receiver/canister/enrollment.rs',s=>s.replace('use ic_platform_runtime::{cdk::api, cdk::call::Call};','use ic_platform_runtime::{system as api, call::{Call, Wait, Outcome as CallOutcome}};').replace('Call::bounded_wait(authority, "authority_fixture_snapshot")\n        .with_raw_args(&bytes)\n        .await','Call::new(authority, "authority_fixture_snapshot", Wait::Bounded { seconds: 300 })\n        .owned_args(bytes).execute()\n        .await').replace('    let snapshot: Snapshot = ic_wire::decode(','    let (outcome, _) = response.into_parts();\n    let CallOutcome::Reply(response) = outcome else { return Err(Refusal::Unavailable); };\n    let snapshot: Snapshot = ic_wire::decode('));
for (const file of ['mod.rs', 'enrollment.rs', 'receiver.rs']) edit(`qualification/ic-auth-host-conformance/tests/installed/${file}`, s => s.replace(/pic\.upgrade_canister\(id, (wasm(?:\.clone\(\))?), vec!\[\], None\)\s*\.await\s*\.unwrap\(\);/g, (_, wasm) => `pic.stop_canister(id, None).await.unwrap();
    assert_eq!(pic.canister_status(id, None).await.unwrap().status, ic_management_canister_types::CanisterStatusType::Stopped);
    pic.upgrade_canister(id, ${wasm}, vec![], None).await.unwrap();
    pic.start_canister(id, None).await.unwrap();`));
for (const name of ['Cargo.toml','qualification/Cargo.toml']) adoptTooling(`${output}/${name}`);
writeFileSync(`${output}/source.json`,JSON.stringify({revision,root,scope:'actual Auth installed fixture with explicit new Runtime adapters; current Platform authentication and construction; no legacy Platform'},null,2)+'\n');
console.log(output);

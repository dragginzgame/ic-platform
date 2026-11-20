// Deliberate fixture generation in an isolated legacy oracle, never a product dependency.
import {mkdtempSync,mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {execFileSync} from 'node:child_process';
mkdirSync('target',{recursive:true});
const out=mkdtempSync(resolve('target/management-retained-'));
mkdirSync(`${out}/src`);
writeFileSync(`${out}/src/main.rs`,readFileSync('qualification/management-retained/oracle.rs'));
writeFileSync(`${out}/Cargo.lock`,readFileSync('qualification/management-retained/oracle.lock'));
writeFileSync(`${out}/Cargo.toml`, `[package]
name = "management-retained-oracle"
version = "0.1.0"
edition = "2024"
publish = false
[workspace]
[dependencies]
ic-platform = { version = "=0.33.7", default-features = false, features = ["management-canister-types", "cycles-ledger-types"] }
`);
execFileSync('cargo',['run','--locked','--manifest-path',`${out}/Cargo.toml`],{stdio:'inherit'});
console.log('Review the root-*.bin diff; then run make qualify-management-ledger qualify-management-snapshots.');

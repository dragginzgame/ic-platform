// Operator wrapper around icp; no callback persistence or runtime recovery policy.
import { parseArgs } from 'node:util';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';

let stage = 'preflight';
try {
  const { values: v } = parseArgs({ options: {
    network: { type: 'string' }, identity: { type: 'string' }, canister: { type: 'string' },
    wasm: { type: 'string' }, 'argument-file': { type: 'string' },
    'timeout-seconds': { type: 'string', default: '300' },
    execute: { type: 'boolean' }, help: { type: 'boolean' },
  } });
  if (v.help) {
    console.log('Usage: node scripts/upgrade-canister.mjs --network NETWORK --identity IDENTITY --canister PRINCIPAL --wasm FILE [--argument-file HEX_FILE] [--timeout-seconds 300] [--execute]');
    console.log('Default: preview only. Execute: stop, confirm Stopped, upgrade, verify module, start, confirm Running.');
    process.exit(0);
  }
  for (const key of ['network', 'identity', 'canister', 'wasm']) {
    if (!v[key] || v[key].startsWith('-')) throw new Error(`Explicit --${key} is required`);
  }
  if (!/^(?:[a-z2-7]{5}-)+[a-z2-7]{1,5}$/.test(v.canister)) throw new Error('Use an explicit canister principal, not a project alias');
  const seconds = Number(v['timeout-seconds']);
  if (!Number.isInteger(seconds) || seconds < 1 || seconds > 3600) throw new Error('Timeout must be 1..3600 seconds per icp command');
  const wasm = resolve(v.wasm);
  const bytes = readFileSync(wasm);
  if (!bytes.subarray(0, 8).equals(Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]))) throw new Error('Supply an uncompressed Wasm module');
  const hash = createHash('sha256').update(bytes).digest('hex');
  const argument = v['argument-file'] ? readFileSync(resolve(v['argument-file']), 'utf8').trim() : '';
  if (!/^(?:[a-fA-F0-9]{2})*$/.test(argument)) throw new Error('Argument file must contain raw argument bytes encoded as hexadecimal');
  const command = (verb, extra = []) => ['canister', verb, v.canister, '--identity', v.identity, '--network', v.network, ...extra];
  const install = command('install', ['--mode', 'upgrade', '--wasm', wasm, '--args-format', 'hex', '--args', argument]);
  console.log(`Target: ${v.canister} | network: ${v.network} | identity: ${v.identity}`);
  console.log(`Artifact: ${wasm}\nWasm SHA-256: ${hash}\nUpgrade argument: ${argument.length / 2} bytes`);
  console.log('Workflow: status → stop/drain → Stopped → upgrade → verify module → start → Running');
  if (!v.execute) {
    console.log('Preview only; no icp commands executed. Add --execute to apply this workflow.');
    process.exit(0);
  }
  async function run(args, quiet = false) {
    return new Promise((resolveResult, reject) => {
      const child = spawn('icp', args, { stdio: ['inherit', 'pipe', 'inherit'], shell: false });
      let output = '', timedOut = false, oversized = false;
      const timeout = setTimeout(() => { timedOut = true; child.kill('SIGKILL'); }, seconds * 1000);
      const began = Date.now();
      const progress = stage === 'stop/drain' ? setInterval(() => {
        console.log(`Waiting for stop to finish (${Math.floor((Date.now() - began) / 1000)}s). Outstanding calls may still be draining; no upgrade has started.`);
      }, 5000) : undefined;
      child.stdout.on('data', data => {
        output += data.toString();
        if (!quiet) process.stdout.write(data);
        if (output.length > 1024 * 1024) { oversized = true; child.kill('SIGKILL'); }
      });
      child.on('error', reject);
      child.on('close', code => {
        clearTimeout(timeout); clearInterval(progress);
        if (timedOut) reject(new Error('icp timed out; its remote operation may still be progressing'));
        else if (oversized) reject(new Error('icp output exceeded the bounded capture limit'));
        else if (code !== 0) reject(new Error(`icp failed with exit code ${code}`));
        else resolveResult(output);
      });
    });
  }
  async function status(expected, verifyHash = false) {
    const output = await run(command('status', ['--json']), true);
    const result = JSON.parse(output);
    if (result?.id !== v.canister || !['Running', 'Stopping', 'Stopped'].includes(result.status)) {
      throw new Error('Unrecognized or incomplete icp status output; refusing to proceed');
    }
    const state = result.status;
    console.log(`Observed status: ${state}`);
    if (expected && state !== expected) throw new Error(`Required ${expected}, observed ${state}; refusing to proceed`);
    if (verifyHash && result.module_hash !== `0x${hash}`) throw new Error('Installed module hash does not match the selected artifact');
    return state;
  }
  const initial = await status();
  if (initial !== 'Stopped') {
    stage = 'stop/drain';
    console.log('Requesting stop. Stopping is not Stopped; callbacks must finish first.');
    await run(command('stop'));
  }
  stage = 'confirm stopped';
  await status('Stopped');
  // Refuse a changed artifact rather than installing bytes other than the previewed hash.
  if (createHash('sha256').update(readFileSync(wasm)).digest('hex') !== hash) throw new Error('Wasm changed during draining');
  stage = 'upgrade';
  console.log('Stopped confirmed. Installing in upgrade mode.');
  await run(install);
  stage = 'verify upgrade';
  await status('Stopped', true);
  stage = 'start';
  console.log('Module verified. Starting canister.');
  await run(command('start'));
  stage = 'verify running';
  await status('Running', true);
  console.log('Upgrade complete. Run the application-specific health check.');
} catch (error) {
  console.error(`Upgrade halted during ${stage}: ${error.message}`);
  console.error('No further install/start commands will run. Inspect canister status before recovery; it may be stopping or stopped. A timeout does not cancel an on-chain operation.');
  process.exitCode = 1;
}

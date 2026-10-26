import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdir, mkdtemp, readFile, writeFile, copyFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { promisify, parseArgs } from 'node:util';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { build } from '../../packages/browser/node_modules/esbuild/lib/main.js';
import { checkBrowserSdkGraph } from './browser-sdk-graph.mjs';
import { checkIcSdkBoundary } from './check-ic-sdk-boundary.mjs';

const { values } = parseArgs({ options: { browser: { type: 'boolean', default: false } }, allowPositionals: false });
const root = fileURLToPath(new URL('../..', import.meta.url));
const source = resolve(root, 'packages/browser');
const sourceManifest = JSON.parse(await readFile(resolve(source, 'package.json'), 'utf8'));
const sdkVersion = sourceManifest.dependencies['@icp-sdk/core'];
await mkdir(resolve(root, 'target'), { recursive: true });
const fixture = await mkdtemp(resolve(root, 'target/browser-consumer.'));
console.log(`Browser consumer evidence: ${fixture}`);
const exec = promisify(execFile);
let commandIndex = 0;
async function command(file, args, cwd = fixture) {
  const log = resolve(fixture, `${++commandIndex}-${file}.log`);
  try {
    // An outer npm publication dry-run must still create local test archives
    // and install disposable consumers. No command here publishes anything.
    const result = await exec(file, args, { cwd, maxBuffer: 4 * 1024 * 1024,
      env: { ...process.env, npm_config_dry_run: 'false' } });
    await writeFile(log, result.stdout + result.stderr);
    return result.stdout;
  } catch (error) {
    await writeFile(log, (error.stdout ?? '') + (error.stderr ?? ''));
    throw error;
  }
}
const packed = JSON.parse(await command('npm', ['pack', '--offline', '--ignore-scripts', '--json', '--pack-destination', fixture], source))['@tokostudios/ic-platform'];
assert(packed.files.every(file => /^(dist\/.*\.(js|ts)|package\.json|README\.md|LICENSE)$/.test(file.path)));
const reviewedLock = JSON.parse(await readFile(resolve(source, 'package-lock.json'), 'utf8'));
async function installConsumer(manifest) {
  // Seed the qualified selections so a fresh npm-ci cache needs tarballs only,
  // not unprepared registry metadata. Only this disposable lock is rewritten.
  const lock = structuredClone(reviewedLock);
  lock.name = manifest.name;
  lock.version = '0.0.0';
  lock.packages[''] = { name: manifest.name, version: '0.0.0', dependencies: manifest.dependencies };
  lock.packages['node_modules/@tokostudios/ic-platform'] = {
    version: packed.version, resolved: `file:${resolve(fixture, packed.filename)}`,
    integrity: packed.integrity, dependencies: { '@icp-sdk/core': sdkVersion },
  };
  await writeFile(resolve(fixture, 'package.json'), JSON.stringify({ ...manifest, version: '0.0.0' }));
  await writeFile(resolve(fixture, 'package-lock.json'), JSON.stringify(lock));
  await command('npm', ['install', '--offline', '--package-lock-only', '--ignore-scripts', '--no-audit', '--no-fund']);
  await command('npm', ['ci', '--offline', '--ignore-scripts', '--no-audit', '--no-fund']);
  const selected = JSON.parse(await readFile(resolve(fixture, 'package-lock.json'), 'utf8'));
  for (const [path, dependency] of Object.entries(selected.packages)) {
    if (path === '' || path === 'node_modules/@tokostudios/ic-platform') continue;
    assert.equal(dependency.version, reviewedLock.packages[path]?.version, `Unreviewed selection: ${path}`);
    assert.equal(dependency.integrity, reviewedLock.packages[path]?.integrity, `Changed source: ${path}`);
  }
}
await installConsumer({
  name: 'platform-browser-consumer-proof', private: true, type: 'module',
  dependencies: { '@tokostudios/ic-platform': `file:${resolve(fixture, packed.filename)}` },
});
const graph = JSON.parse(await command('npm', ['ls', '--all', '--omit=dev', '--json']));
const supplier = graph.dependencies['@tokostudios/ic-platform'];
assert.equal(supplier.version, packed.version);
assert.equal(supplier.dependencies['@icp-sdk/core'].version, sdkVersion);
await checkBrowserSdkGraph(fixture, sdkVersion, ['@tokostudios/ic-platform']);
assert.equal(await readFile(resolve(fixture, 'node_modules/@tokostudios/ic-platform/LICENSE'), 'utf8'),
  await readFile(resolve(root, 'LICENSE'), 'utf8'), 'Packed license differs from its owner');
await copyFile(resolve(source, 'tests/consumer.ts'), resolve(fixture, 'consumer.ts'));
await checkIcSdkBoundary([resolve(fixture, 'package.json'), resolve(fixture, 'consumer.ts')]);
const configuration = JSON.parse(await readFile(resolve(source, 'tsconfig.json'), 'utf8'));
configuration.compilerOptions.rootDir = '.';
configuration.compilerOptions.outDir = 'compiled';
configuration.include = ['consumer.ts'];
await writeFile(resolve(fixture, 'tsconfig.json'), JSON.stringify(configuration));
await command('node', [resolve(source, 'node_modules/typescript/lib/tsc.js'), '-p', 'tsconfig.json']);
const { smoke } = await import(pathToFileURL(resolve(fixture, 'compiled/consumer.js')));
assert.equal((await smoke()).calls, 1);
// Export-map admission: consumers cannot reach internal SDK plumbing.
await writeFile(resolve(fixture, 'private-import.mjs'), "import '@tokostudios/ic-platform/dist/identity-import.js';\n");
await assert.rejects(exec(process.execPath, ['private-import.mjs'], { cwd: fixture }), error => error.stderr.includes('ERR_PACKAGE_PATH_NOT_EXPORTED'));
const bundled = await build({ entryPoints: [resolve(fixture, 'consumer.ts')], bundle: true,
  format: 'esm', platform: 'browser', target: 'es2022', metafile: true,
  outfile: resolve(fixture, 'bundle.js') });
await writeFile(resolve(fixture, 'bundle-meta.json'), JSON.stringify(bundled.metafile, null, 2));
const principal = await build({ stdin: { contents: "export { parseCanonicalPrincipal } from '@tokostudios/ic-platform/principal';", resolveDir: fixture },
  bundle: true, format: 'esm', platform: 'browser', metafile: true, write: false });
assert(!Object.keys(principal.metafile.inputs).some(path => /\/core\/lib\/esm\/(agent|candid|identity)\//.test(path)));
console.log('Packed-only Node/TypeScript consumer, export isolation and browser bundles passed');

// Providers remain application dependencies. Verify their real Agent/Identity
// types share the selected SDK; do not turn them into production Platform deps.
const providerManifest = {
  name: 'platform-browser-consumer-proof', private: true, type: 'module',
  dependencies: {
    '@tokostudios/ic-platform': `file:${resolve(fixture, packed.filename)}`,
    '@caffeineai/object-storage': sourceManifest.devDependencies['@caffeineai/object-storage'],
    '@icp-sdk/auth': sourceManifest.devDependencies['@icp-sdk/auth'],
  },
  // Caffeine still declares SDK ^5.3.0. This is a qualified application override,
  // not provider-declared compatibility or an override inherited from Platform.
  overrides: { '@caffeineai/object-storage': { '@icp-sdk/core': sdkVersion } },
};
await copyFile(resolve(fixture, 'package.json'), resolve(fixture, 'platform-only.package.json'));
await copyFile(resolve(fixture, 'package-lock.json'), resolve(fixture, 'platform-only.lock.json'));
await installConsumer(providerManifest);
await copyFile(resolve(source, 'tests/interop.ts'), resolve(fixture, 'interop.ts'));
await checkIcSdkBoundary([resolve(fixture, 'package.json'), resolve(fixture, 'consumer.ts'), resolve(fixture, 'interop.ts')]);
configuration.include.push('interop.ts');
await writeFile(resolve(fixture, 'tsconfig.json'), JSON.stringify(configuration));
await command('node', [resolve(source, 'node_modules/typescript/lib/tsc.js'), '-p', 'tsconfig.json']);
const providerGraph = JSON.parse(await command('npm', ['ls', '@icp-sdk/core', '--all', '--json']));
function checkSdk(node) {
  for (const [name, dependency] of Object.entries(node.dependencies ?? {})) {
    if (name === '@icp-sdk/core') assert.equal(dependency.version, sdkVersion);
    checkSdk(dependency);
  }
}
checkSdk(providerGraph);
await checkBrowserSdkGraph(fixture, sdkVersion,
  ['@tokostudios/ic-platform', '@caffeineai/object-storage', '@icp-sdk/auth', '@icp-sdk/signer']);
console.log(`Caffeine Agent/Auth Identity type compatibility and shared SDK ${sdkVersion} graph passed`);

if (values.browser) {
  const { chromium } = await import('../../packages/browser/node_modules/playwright/index.mjs');
  const bundle = await readFile(resolve(fixture, 'bundle.js'));
  const server = createServer((request, response) => {
    response.setHeader('Content-Type', 'text/javascript');
    if (request.url === '/bundle.js') response.end(bundle);
    else if (request.url === '/worker.js') response.end("import { smoke } from '/bundle.js'; smoke().then(value => postMessage({value}), error => postMessage({error:String(error)}));");
    else { response.setHeader('Content-Type', 'text/html'); response.end('<!doctype html><title>Platform consumer</title>'); }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    const result = await page.evaluate(async () => {
      const { smoke } = await import('/bundle.js');
      const main = await smoke();
      const worker = await new Promise((resolve, reject) => {
        const worker = new Worker('/worker.js', { type: 'module' });
        const timer = setTimeout(() => { worker.terminate(); reject(new Error('Worker deadline')); }, 10_000);
        worker.onerror = error => { clearTimeout(timer); worker.terminate(); reject(new Error(error.message)); };
        worker.onmessage = ({ data }) => { clearTimeout(timer); worker.terminate(); data.error ? reject(new Error(data.error)) : resolve(data.value); };
      });
      return { main, worker };
    });
    assert.deepEqual(result.main, result.worker);
    await writeFile(resolve(fixture, 'chromium.json'), JSON.stringify({ version: browser.version(), result }, null, 2));
    console.log('Actual Chromium page and module-worker SDK qualification passed');
  } finally {
    await browser?.close();
    await new Promise(resolve => server.close(resolve));
  }
}

import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

// Shared admission for explicit setup and offline validation; installs nothing.
export const compilerFloor = JSON.parse(await readFile(
  new URL('../fixtures/browser-compiler-floor/package.json', import.meta.url), 'utf8'));
assert.equal(process.versions.node, compilerFloor.engines.node, 'Use the pinned contributor Node version');
assert.equal(execFileSync('npm', ['--version'], { encoding: 'utf8' }).trim(), compilerFloor.engines.npm,
  'Use the pinned contributor npm version');

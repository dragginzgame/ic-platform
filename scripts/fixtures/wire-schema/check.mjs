import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { principal } from './principal.js';
const vectors = JSON.parse(readFileSync(new URL('./vectors.json', import.meta.url)));
for (const { raw, request, response } of vectors.valid) {
  const value = { caller: Uint8Array.from(raw) };
  assert.deepEqual([...principal.encodeRequest(value)], request);
  assert.deepEqual([...principal.encodeResponse(value)], response);
  assert.deepEqual(principal.decodeRequest(Uint8Array.from(request)).caller, value.caller);
  assert.deepEqual(principal.decodeResponse(Uint8Array.from(response)).caller, value.caller);
}
for (const { raw, request, response } of vectors.invalid) {
  const value = { caller: Uint8Array.from(raw) };
  assert.throws(() => principal.encodeRequest(value));
  assert.throws(() => principal.encodeResponse(value));
  assert.throws(() => principal.decodeRequest(Uint8Array.from(request)));
  assert.throws(() => principal.decodeResponse(Uint8Array.from(response)));
}
console.log('Principal Rust/TypeScript frames and length refusals passed');

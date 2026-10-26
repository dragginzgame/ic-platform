import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { importIdentity } from './identity-import.js';

export { Ed25519KeyIdentity };
export { IdentityImportError, type IdentityImportCode } from './identity-import.js';

/** Restore an SDK identity after checking its advertised key pair. No key is generated. */
export function restoreEd25519Identity(json: string, maxJsonBytes: number): Ed25519KeyIdentity {
  return importIdentity(Ed25519KeyIdentity, json, maxJsonBytes);
}

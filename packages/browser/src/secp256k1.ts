import { Secp256k1KeyIdentity } from '@icp-sdk/core/identity/secp256k1';
import { importIdentity } from './identity-import.js';

export { Secp256k1KeyIdentity };
export { IdentityImportError, type IdentityImportCode } from './identity-import.js';

/** Restore through the unchanged SDK implementation; callers select the import budget. */
export function restoreSecp256k1Identity(json: string, maxJsonBytes: number): Secp256k1KeyIdentity {
  return importIdentity(Secp256k1KeyIdentity, json, maxJsonBytes);
}

// Generate the distribution copy from the sole maintained repository licence.
import { copyFile, mkdir } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const dist = new URL('packages/browser/dist/', root);
await mkdir(dist, { recursive: true });
await copyFile(new URL('LICENSE.md', root), new URL('LICENSE.md', dist));

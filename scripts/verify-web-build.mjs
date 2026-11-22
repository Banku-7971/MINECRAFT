// Vercel build step. The game is compiled locally and committed as
// web/rustcraft.wasm, so the deploy only needs to verify the artifact exists.
// This keeps Vercel builds fast and avoids installing a Rust toolchain there.
import { statSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const wasm = resolve(root, 'web', 'rustcraft.wasm');

let size = 0;
try {
  size = statSync(wasm).size;
} catch {
  // handled below
}

if (size < 100_000) {
  console.error('web/rustcraft.wasm is missing or incomplete.');
  console.error('Run `npm run build:wasm` before committing.');
  process.exit(1);
}

console.log(`Static WebAssembly site ready (${Math.round(size / 1024)} KB).`);

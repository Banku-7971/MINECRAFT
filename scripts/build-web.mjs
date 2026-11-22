// Build the browser game: compile the library to WebAssembly and copy the
// result into web/ so Vercel can serve it as a static file.
import { spawnSync } from 'child_process';
import { copyFileSync, mkdirSync, statSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const targetDir = process.env.CARGO_TARGET_DIR || resolve(root, 'target');
const cargo = process.env.CARGO || 'cargo';

mkdirSync(targetDir, { recursive: true });

console.log('Building WebAssembly with ' + cargo + '...');
const result = spawnSync(
  cargo,
  ['build', '--release', '--target', 'wasm32-unknown-unknown', '--lib'],
  {
    cwd: root,
    stdio: 'inherit',
    env: { ...process.env, CARGO_TARGET_DIR: targetDir },
  },
);

if (result.error) {
  console.error(`Could not run ${cargo}: ${result.error.message}`);
  console.error('Install Rust and the wasm target, then try again:');
  console.error('  rustup target add wasm32-unknown-unknown');
  process.exit(1);
}
if (result.status !== 0) process.exit(result.status ?? 1);

const built = resolve(targetDir, 'wasm32-unknown-unknown', 'release', 'rustcraft.wasm');
const output = resolve(root, 'web', 'rustcraft.wasm');
copyFileSync(built, output);

const kb = Math.round(statSync(output).size / 1024);
console.log(`Copied web/rustcraft.wasm (${kb} KB)`);

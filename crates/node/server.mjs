import { readFile } from 'node:fs/promises';
import process from 'node:process';
import init, { Miasma } from './build/miasma_node.js';
import { createServer } from './adapter.mjs';

await init({ module_or_path: await readFile(new URL('./build/miasma_node_bg.wasm', import.meta.url)) });
const miasma = new Miasma(process.env.POISON_SOURCE ?? '', 8);
createServer(request => miasma.fetch(request)).listen(
  Number(process.env.PORT ?? 9999), process.env.HOST ?? '127.0.0.1',
);

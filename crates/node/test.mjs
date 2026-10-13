import assert from 'node:assert/strict';
import { once } from 'node:events';
import { readFile } from 'node:fs/promises';
import { createServer as httpServer, request } from 'node:http';
import { gzipSync, gunzipSync } from 'node:zlib';
import init, { Miasma } from './build/miasma_node.js';
import { createServer } from './adapter.mjs';

const servers = [];
const timeout = setTimeout(() => { throw new Error('Adapter checks timed out'); }, 20_000);
async function listen(server) {
  servers.push(server);
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  return `http://127.0.0.1:${server.address().port}`;
}
function get(url, headers = {}, onChunk = () => {}) {
  return new Promise((resolve, reject) => {
    request(url, { headers }, response => {
      const chunks = [];
      response.on('data', chunk => { chunks.push(chunk); onChunk(chunk); });
      response.on('end', () => resolve({ response, body: Buffer.concat(chunks) }));
      response.on('error', reject);
    }).on('error', reject).end();
  });
}

try {
  const echo = await listen(createServer(async req => new Response(await req.text(), {
    status: 201, headers: { 'x-method': req.method, 'x-path': new URL(req.url).pathname },
  })));
  const echoed = await fetch(`${echo}/echo`, { method: 'POST', body: 'streamed request' });
  assert.equal(echoed.status, 201);
  assert.equal(echoed.headers.get('x-method'), 'POST');
  assert.equal(echoed.headers.get('x-path'), '/echo');
  assert.equal(await echoed.text(), 'streamed request');

  const empty = await listen(createServer(() => new Response(null, { status: 204 })));
  assert.equal((await fetch(empty)).status, 204);
  const originalError = console.error;
  console.error = () => {};
  try {
    const broken = await listen(createServer(() => { throw new Error('test'); }));
    assert.equal((await fetch(broken)).status, 500);
    const locked = await listen(createServer(() => {
      const response = new Response('locked');
      response.body.getReader();
      return response;
    }));
    await assert.rejects(get(locked));
  } finally { console.error = originalError; }

  const cancelled = Promise.withResolvers();
  const received = Promise.withResolvers();
  const disconnected = await listen(createServer(async req => {
    received.resolve();
    await new Promise(resolve => req.signal.addEventListener('abort', resolve, { once: true }));
    return new Response(new ReadableStream({ cancel: () => cancelled.resolve() }));
  }));
  const client = request(disconnected);
  client.on('error', () => {});
  client.end();
  await received.promise;
  client.destroy();
  await cancelled.promise;

  await init({ module_or_path: await readFile(new URL('./build/miasma_node_bg.wasm', import.meta.url)) });
  assert.throws(() => new Miasma('invalid URL', 1));
  assert.throws(() => new Miasma('', 0xffff_ffff));
  const marker = '<node-poison>&'.repeat(2048);
  let upstreamRequests = 0;
  const release = Promise.withResolvers();
  const upstream = await listen(httpServer(async (req, res) => {
    assert.match(req.headers['user-agent'], /^miasma\//);
    upstreamRequests++;
    await release.promise;
    res.writeHead(200, { 'content-encoding': 'gzip' }).end(gzipSync(marker));
  }));
  const miasma = new Miasma(upstream, 1);
  const url = await listen(createServer(req => miasma.fetch(req)));
  let streamed = false;
  const plain = await get(url, {}, () => { streamed = true; release.resolve(); });
  assert.ok(streamed);
  const escaped = '&lt;node-poison&gt;&amp;';
  assert.ok(plain.body.toString().includes(escaped.repeat(2048)));
  assert.ok(/href="\/[0-9a-f-]{36}"/.test(plain.body.toString()));
  const compressed = await get(url, { 'accept-encoding': 'gzip' });
  assert.equal(compressed.response.headers['content-encoding'], 'gzip');
  assert.ok(gunzipSync(compressed.body).toString().includes(escaped.repeat(2048)));
  assert.ok(upstreamRequests > 0);

  const dropped = Promise.withResolvers();
  const disconnectUrl = await listen(createServer(async req => {
    const response = await miasma.fetch(req);
    req.signal.addEventListener('abort', () => dropped.resolve(), { once: true });
    return response;
  }));
  request(disconnectUrl, res => res.once('data', () => res.destroy()))
    .on('error', () => {}).end();
  await dropped.promise;
  assert.equal((await get(url)).response.statusCode, 200);

  const held = await miasma.fetch(new Request(url));
  assert.equal((await miasma.fetch(new Request(url))).status, 429);
  await held.body.cancel();
  const recovered = await miasma.fetch(new Request(url));
  assert.equal(recovered.status, 200);
  await recovered.body.cancel();
  miasma.free();
  console.log('Adapter checks passed');
} finally {
  clearTimeout(timeout);
  await Promise.all(servers.map(server => new Promise(resolve => {
    server.close(resolve);
    server.closeAllConnections();
  })));
}

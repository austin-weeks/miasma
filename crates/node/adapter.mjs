import { createServer as httpServer } from 'node:http';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';

export function createServer(fetch) {
  return httpServer(async (incoming, outgoing) => {
    const abort = new AbortController();
    outgoing.once('close', () => abort.abort());
    try {
      const response = await fetch(new Request(new URL(incoming.url, 'http://localhost'), {
        method: incoming.method,
        headers: incoming.headers,
        signal: abort.signal,
        ...(incoming.method === 'GET' || incoming.method === 'HEAD' ? {} : {
          body: Readable.toWeb(incoming), duplex: 'half',
        }),
      }));
      if (outgoing.destroyed) {
        await response.body?.cancel();
        return;
      }
      outgoing.writeHead(response.status, Object.fromEntries(response.headers));
      if (response.body) {
        await pipeline(Readable.fromWeb(response.body), outgoing);
      } else {
        outgoing.end();
      }
    } catch (error) {
      if (abort.signal.aborted) return;
      console.error(error);
      if (outgoing.headersSent) outgoing.destroy();
      else outgoing.writeHead(500).end();
    }
  });
}

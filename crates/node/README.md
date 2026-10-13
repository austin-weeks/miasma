# Node.js adapter

Streams Miasma through `node:http` with gzip, backpressure, and disconnect
cancellation. Also runs on Bun and Deno.

Install Rust and `wasm-pack`, then build from this directory:

```sh
cargo install wasm-pack --locked
wasm-pack build --no-opt --profile wasm-release --target web --out-dir build -- --locked
```

Run:

```sh
node server.mjs
bun server.mjs
deno run --allow-read --allow-net --allow-env server.mjs
```

Listens on `127.0.0.1:9999`. Set `HOST`, `PORT`, or `POISON_SOURCE` to override.
The server reuses its router and cache with up to eight concurrent requests.
SQLite metrics and configuration files are unavailable on WASM.

Run `node test.mjs`, `bun test.mjs`, or
`deno run --allow-read --allow-net --allow-env test.mjs` to check the adapter.

To embed it, initialize `build/miasma_node.js`, construct `Miasma(source, limit)`,
and call `fetch(Request)`. Use `adapter.mjs` to serve HTTP.

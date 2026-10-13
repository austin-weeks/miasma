# Cloudflare Worker adapter

Streams Miasma through Cloudflare's Fetch handler. Each isolate reuses its
router and poison cache, with up to eight concurrent requests.

Install Rust with `wasm32-unknown-unknown`, Node.js, and `worker-build`:

```sh
rustup target add wasm32-unknown-unknown
cargo install worker-build --version 0.8.7 --locked
```

From this directory, run locally:

```sh
npm exec --yes --package wrangler@4.149.0 -- wrangler dev --local
```

Set `POISON_SOURCE` in `wrangler.toml`, or pass
`--var POISON_SOURCE:https://your-source.example/` to Wrangler.

The Python 3 smoke test checks upstream and response gzip, HTML escaping,
and generated links, then stops both servers:

```sh
python3 smoke.py
```

Fetch decompresses upstream gzip; `EncodeBody::Manual` prevents double
compression of Miasma's gzipped responses. After five consecutive fetch
failures, the breaker allows one recovery probe every 20 seconds. Cache and
breaker state last only for the isolate's lifetime.

The Free plan's CPU budget excludes network waiting but includes generation
and compression. Measure deployed CPU usage for your poison source and load.

Requires Fetch, Web Crypto, timers, and one JavaScript thread on
`wasm32-unknown-unknown`. SQLite metrics, the native CLI, and configuration
files are unavailable.

Cloudflare's [Worker metrics](https://developers.cloudflare.com/workers/observability/metrics-and-analytics/)
record requests and runtime errors automatically. Handled upstream failures
that serve cached or fallback data count as successful invocations.

Miasma supports gzip compression. Cloudflare also supports
[Brotli and Zstandard at the edge](https://developers.cloudflare.com/speed/optimization/content/compression/);
Miasma does not encode either format.

Deploy to your Cloudflare account:

```sh
npm exec --yes --package wrangler@4.149.0 -- wrangler deploy
```

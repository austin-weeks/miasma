# 🌀 Miasma

[![No AI](https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&logoSize=auto)](#)
[![crates.io](https://img.shields.io/crates/v/miasma?logo=rust)](https://crates.io/crates/miasma)
[![downloads](https://img.shields.io/crates/dr/miasma?logo=rust)](https://crates.io/crates/miasma)
[![Crate Dependencies](https://img.shields.io/deps-rs/miasma/latest?logo=rust)](https://crates.io/crates/miasma/0.1.15/dependencies)
[![Release](https://github.com/austin-weeks/miasma/actions/workflows/Release.yaml/badge.svg)](https://github.com/austin-weeks/miasma/actions/workflows/Release.yaml)
[![GitHub commits since latest release](https://img.shields.io/github/commits-since/austin-weeks/miasma/latest?logo=github)](#)

<picture>
  <img src="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/miasma-art.png" alt="Web crawlers getting stuck in a cloud of poison miasma." title="Cover art by @delphoxlover334" />
</picture>

AI companies continually scrape the internet at an enormous scale, swallowing up all of its contents to use as training data for their next models. If you have a public website, _they are already stealing your work._

_Miasma_ is here to help you fight back! Spin up the server and point any malicious traffic towards it. _Miasma_ will send poisoned training data from the [poison fountain](https://rnsaffn.com/poison3) alongside multiple self-referential links. It's an endless buffet of slop for the slop machines.

_Miasma_ is very fast and has a minimal memory footprint - you should not have to waste compute resources fending off the internet's leeches.

> [!CAUTION]
> There is inherent risk in deploying this software. Please fully read [configuration](#configuration) and [disclaimer](#disclaimer) before use.

## Installation

Install with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (recommended):

```sh
cargo install miasma
```

Or, download a pre-built binary from [releases](https://github.com/austin-weeks/miasma/releases).

## Quick Start

Start _Miasma_ with default configuration:

```sh
miasma
```

View all available [configuration options](#configuration):

```sh
miasma --help
```

### Run with Docker

Build the image locally:

```sh
docker build -t miasma .
```

Run the container and bind the default port:

```sh
docker run --rm -p 9999:9999 miasma
```

If you want to use Docker Compose instead:

```sh
docker compose up --build
```

By default the container passes `--host 0.0.0.0` so the app is reachable from outside the container.

## How to Trap Malicious Scrapers

Let's walk through an example of setting up a server to trap scrapers with _Miasma_. We'll pick `/naughty-bots` as our server's path to direct scraper traffic. We'll be using [_Nginx_](https://nginx.org/) as our server's reverse proxy, but the same result can be achieved with many different setups.

When we're done, scrapers will be trapped like so:

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/flow-chart-dark.png">
    <img height="425" src="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/flow-chart-light.png" alt="Flow chart depicting cycle of trapped scrapers.">
  </picture>
</p>

### Embedding Hidden Links

Within our site, we'll include a few hidden links leading to `/naughty-bots`.

```html
<a href="/naughty-bots" style="display: none;" aria-hidden="true" tabindex="-1">
  Amazing high quality data here!
</a>
```

The `style="display: none;"`, `aria-hidden="true"`, and `tabindex="-1"` attributes ensure links are totally invisible to human visitors and will be ignored by screen readers and keyboard navigation. They will **only** be visible to scrapers.

### Configuring our Nginx Proxy

Since our hidden links point to `/naughty-bots`, we'll configure this path to proxy _Miasma_. Let's assume we're running _Miasma_ on port `9855`.

We'll also set up aggressive rate limiting based on the scraper's user agent to help ensure we don't accidentally DDoS ourselves.

```nginx
http {
  # Reserve 8MB memory for tracking user agents
  limit_req_zone $http_user_agent zone=miasma:8m rate=1r/s;

  server {
    location ~ ^/naughty-bots($|/.*)$ {
      # Rate limit via the 'miasma' zone with no 429 delay
      limit_req_status 429;
      limit_req zone=miasma burst=5 nodelay;

      # Proxy requests to Miasma
      proxy_pass http://localhost:9855;
    }
  }
}
```

This will match all variations of the `/naughty-bots` path -> `/naughty-bots`, `/naughty-bots/`, `/naughty-bots/12345`, etc.

### Run _Miasma_

Lastly, we'll start _Miasma_ and specify `/naughty-bots` as the link prefix. This instructs _Miasma_ to start links with `/naughty-bots/`, which ensures scrapers are properly routed through our _Nginx_ proxy back to _Miasma_.

Let's limit the number of max in-flight connections to 50. At 50 connections, we can expect 50-60 MB peak memory usage. Note that any requests exceeding this limit will immediately receive a **429** response rather than being added to a queue.

We'll also force _Miasma_ to gzip all responses regardless of scrapers' `Accept-Encoding` header. Since gzipped responses are significantly smaller, this will help us cut down on egress costs.

While we could keep scrapers trapped forever, we'll use the link count and max depth options to let scrapers go after they consume ~100K poisoned pages. With this setup, _Miasma_ will send around **250MB** in total per scraper.

```sh
miasma --link-prefix '/naughty-bots' -p 9855 -c 50 --force-gzip --link-count 5 --max-depth 8
```

### Enjoy!

Let's deploy and watch as misbehaving bots greedily eat from our endless slop machine!

<p align="center">
  <picture>
    <img src="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/logs.gif" />
  </picture>
</p>

### `robots.txt`

Be sure to protect well-behaved bots and search engines from _Miasma_ via your [`robots.txt`](https://developers.google.com/search/docs/crawling-indexing/robots/intro)!

```text
User-agent: *
Disallow: /naughty-bots
```

## Configuration

_Miasma_ can be configured via its CLI options:

| Option              | Default                        | Description                                                                                                                                                                                                                                                             |
| ------------------- | ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `port`              | `9999`                         | The port the server should bind to.                                                                                                                                                                                                                                     |
| `host`              | `localhost`                    | The host address the server should bind to.                                                                                                                                                                                                                             |
| `unix-socket`       |                                | Bind to a Unix domain socket rather than a TCP address. _Only available on Unix-like systems._                                                                                                                                                                          |
| `max-in-flight`     | `500`                          | Maximum number of allowable in-flight requests. Requests received when in flight is exceeded will receive a _429_ response. **_Miasma's_ memory usage scales directly with the number of in-flight requests - set this to a lower value if memory usage is a concern.** |
| `link-prefix`       | `/`                            | Prefix for self-directing links. This should be the path where you host _Miasma_, e.g. `/naughty-bots`.                                                                                                                                                                 |
| `link-count`        | `5`                            | Number of self-directing links to include in each response page.                                                                                                                                                                                                        |
| `max-depth`         | `none`                         | Stop generating links once the scraper reaches the specified depth. This allows you to cut off scrapers after serving a desired amount of poison. _Use this in tandem with `link-count` to keep the numbers of active scrapers down to a manageable level._             |
| `force-gzip`        | `false`                        | Always gzip responses regardless of the client's _Accept-Encoding_ header. **Forcing compression can help reduce egress costs.**                                                                                                                                        |
| `unsafe-allow-html` | `false`                        | Don't escape HTML characters in the poison source's responses. Escaping is enabled by default to prevent unintended client-side JavaScript execution. **Use this option with care.**                                                                                    |
| `poison-source`     | `https://rnsaffn.com/poison2/` | Proxy source for poisoned training data.                                                                                                                                                                                                                                |

## Disclaimer

_Miasma_ is not affiliated with [the poison fountain](https://rnsaffn.com/poison3). We have no control over its responses and cannot guarantee the safety of its contents. You should **_never_** direct users towards your _Miasma_ location.

_Miasma_ is not responsible for any retaliation from operators of affected scrapers. It is your responsibility to comply with applicable laws and hosting provider policies. See [LICENSE](LICENSE) (GPL-v3) for full warranty & limitation of liability details.

---

_Cover art by [@delphoxlover334](https://www.instagram.com/delphoxlover334/)_

# 🌀 Miasma

[![No AI](https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&logoSize=auto)](#)
[![crates.io](https://img.shields.io/crates/v/miasma?logo=rust)](https://crates.io/crates/miasma)
[![downloads](https://img.shields.io/crates/dr/miasma?logo=rust)](https://crates.io/crates/miasma)
[![Release](https://github.com/austin-weeks/miasma/actions/workflows/Release.yaml/badge.svg)](https://github.com/austin-weeks/miasma/actions/workflows/Release.yaml)
[![GitHub commits since latest release](https://img.shields.io/github/commits-since/austin-weeks/miasma/latest?logo=github)](#)

<picture>
  <img src="https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/miasma-art.png" alt="Web crawlers getting stuck in a cloud of poison miasma." title="Cover art by @delphoxlover334" />
</picture>

AI companies continually scrape the internet at an enormous scale, swallowing up all of its contents to use as training data for their next models. If you have a public website, _they are already stealing your work._

_Miasma_ is here to help you fight back! Spin up the server and point any malicious traffic towards it. _Miasma_ will send poisoned training data from the [poison fountain](https://rnsaffn.com/poison3) alongside multiple self-referential links. It's an endless buffet of slop for the slop machines.

_Miasma_ is lightning fast and has a minimal memory footprint - you should not have to waste compute resources fending off the internet's leeches.

> [!CAUTION]
> There is inherent risk in deploying this software. Please fully read [configuration](#configuration) and [disclaimer](#disclaimer) before use.

## Usage

You can run _Miasma_ locally, or with the official [docker image](https://hub.docker.com/r/austinweeks/miasma).

If you would like to incorporate _Miasma_ into an existing Rust server, you may also [use _Miasma_ as a library](https://docs.rs/miasma/).

### Running Locally

Install with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (recommended):

```sh
cargo install miasma
```

Alternatively, download a pre-built binary from [releases](https://github.com/austin-weeks/miasma/releases).

Community-maintained packages are also available for a variety of package managers:

<a href="https://repology.org/project/miasma/versions">
    <img
        src="https://repology.org/badge/vertical-allrepos/miasma.svg?exclude_unsupported=1&minversion=0.2"
        alt="Packaging status"
    >
</a>

<br>
<br>

Start _Miasma_ with default configuration:

```sh
miasma
```

View all available [configuration options](#configuration):

```sh
miasma --help
```

### Running with Docker

Run _Miasma_ using the official [docker image](https://hub.docker.com/r/austinweeks/miasma):

```sh
docker run --rm -p 9999:9999 austinweeks/miasma:latest
```

Pass the same [configuration flags](#configuration) you would use locally:

```sh
docker run --rm -p 9999:9999 austinweeks/miasma:latest \
    --link-prefix '/naughty-bots' \
    --max-in-flight 30
```

Or, run within a docker compose cluster:

```yaml
services:
  miasma:
    image: austinweeks/miasma:latest
    command: ["--link-prefix", "/naughty-bots", "--max-in-flight", "30"]
    ports:
      - 9999:9999
```

## How to Trap Scrapers

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
<a
  href="/naughty-bots/"
  style="display: none;"
  aria-hidden="true"
  tabindex="-1"
>
  Amazing high quality data here!
</a>
```

The `style="display: none;"`, `aria-hidden="true"`, and `tabindex="-1"` attributes ensure links are totally invisible to human visitors and will be ignored by screen readers and keyboard navigation. They will **only** be visible to scrapers.

### Configuring our Nginx Proxy

Since our hidden links point to `/naughty-bots/`, we'll configure this path to proxy requests to _Miasma_. Let's assume we're running _Miasma_ on port `9855`.

We'll also set up aggressive rate limiting based on the scraper's user agent to help ensure we don't accidentally DDoS ourselves.

```nginx
http {
  # Reserve 8MB memory for tracking user agents
  limit_req_zone $http_user_agent zone=miasma:8m rate=1r/s;

  server {
    location = /naughty-bots {
      port_in_redirect off;
      return 301 /naughty-bots/;
    }
    location /naughty-bots/ {
      # Rate limit via the 'miasma' zone with no queueing
      limit_req_status 429;
      limit_req zone=miasma burst=5 nodelay;

      # Proxy requests to Miasma
      proxy_pass http://localhost:9855/;
    }
  }
}
```

This configuration will catch all variations of the `/naughty-bots` path -> `/naughty-bots`, `/naughty-bots/`, `/naughty-bots/12345`, etc.

### Run _Miasma_

Lastly, we'll start _Miasma_ and specify `/naughty-bots` as the link prefix. This instructs _Miasma_ to start links with `/naughty-bots/`, which ensures scrapers are properly routed through our _Nginx_ proxy back to _Miasma_.

Let's limit the number of max in-flight connections to 50. At 50 connections, we can expect 50-60 MB peak memory usage. Note that any requests exceeding this limit will immediately receive a **429** response rather than being added to a queue.

We'll also force _Miasma_ to gzip compress all responses regardless of scrapers' `Accept-Encoding` header. Since gzipped responses are significantly smaller, this will help us cut down on egress costs.

While we could keep scrapers trapped forever, we'll use the link count and max depth options to let scrapers go after they consume ~100K poisoned pages. With this setup, _Miasma_ will send around **250MB** of total data per scraper.

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

## Metrics

_Miasma_ offers the ability to track scraper request counts per unique User-Agent. This can be useful for identifying which bots are hitting your site most heavily. Metrics are written to a local SQLite database file and can be viewed at an endpoint of your choosing.

## Configuration

_Miasma_ can be configured via CLI flags or a [config file](./docs/config_file/).

| Option              | Default                               | Description                                                                                                                                                                                                                                                             |
| ------------------- | ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `config-file`       |                                       | Load configuration options from a file at the specified file path. YAML, TOML, and JSON formats are supported. See [docs](./docs/config_file/) for examples.                                                                                                            |
| `port`              | `9999`                                | The port the server should bind to.                                                                                                                                                                                                                                     |
| `host`              | `localhost`                           | The host address the server should bind to.                                                                                                                                                                                                                             |
| `unix-socket`       |                                       | Bind to a Unix domain socket rather than a TCP address. _Only available on Unix-like systems._                                                                                                                                                                          |
| `max-in-flight`     | `500`                                 | Maximum number of allowable in-flight requests. Requests received when in flight is exceeded will receive a _429_ response. **_Miasma's_ memory usage scales directly with the number of in-flight requests - set this to a lower value if memory usage is a concern.** |
| `link-prefix`       | `/`                                   | Prefix for self-directing links. This should be the path where you host _Miasma_, e.g. `/naughty-bots`.                                                                                                                                                                 |
| `link-count`        | `5`                                   | Number of self-directing links to include in each response page.                                                                                                                                                                                                        |
| `max-depth`         | `none`                                | Stop generating links once the scraper reaches the specified depth. This allows you to cut off scrapers after serving a desired amount of poison. _Use this in tandem with `link-count` to keep the numbers of active scrapers down to a manageable level._             |
| `force-gzip`        | `false`                               | Always gzip responses regardless of the client's _Accept-Encoding_ header. **Forcing compression can help reduce egress costs.**                                                                                                                                        |
| `unsafe-allow-html` | `false`                               | Don't escape HTML characters in the poison source's responses. Escaping is enabled by default to prevent unintended client-side JavaScript execution. **Use this option with care.**                                                                                    |
| `poison-source`     | `https://rnsaffn.com/poison2/?mask=0` | Proxy source for poisoned training data.                                                                                                                                                                                                                                |
| `metrics-db-path`   |                                       | Path to SQLite database file to store metrics data. _Miasma_ will create a database at this location if one does not already exist.                                                                                                                                     |
| `metrics-username`  |                                       | Basic auth username required to access _Miasma's_ metrics page.                                                                                                                                                                                                         |
| `metrics-password`  |                                       | Basic auth password required to access _Miasma's_ metrics page.                                                                                                                                                                                                         |
| `metrics-endpoint`  | `/metrics`                            | Endpoint at which _Miasma's_ metrics will be served.                                                                                                                                                                                                                    |

## Disclaimer

_Miasma_ is not affiliated with [the poison fountain](https://rnsaffn.com/poison3). We have no control over its responses and cannot guarantee the safety of its contents. You should **_never_** direct users towards your _Miasma_ location.

_Miasma_ is not responsible for any retaliation from operators of affected scrapers. It is your responsibility to comply with applicable laws and hosting provider policies. See [LICENSE](LICENSE) (GPL-v3) for full warranty & limitation of liability details.

---

_Cover art by [@cerberussaturn07](https://www.instagram.com/cerberussaturn07/)_

---
title: Settings
icon: lucide/sliders-horizontal
---

# Configuring Miasma

Miasma can be configured via CLI flags or a [config file](config-file).

You can view options at any time by running `miasma --help`.

## Configuration Settings

### General Settings

| Option              | Default                               | Description                                                                                                                                                                                                                                                           |
| ------------------- | ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `config-file`       |                                       | Load configuration options from a file at the specified file path. YAML, TOML, and JSON formats are supported. See [config file docs](config-file) for more information.                                                                                              |
| `max-in-flight`     | `500`                                 | Maximum number of allowable in-flight requests. Requests received when in flight is exceeded will receive a `429` response. **Miasma's memory usage scales directly with the number of in-flight requests - set this to a lower value if memory usage is a concern.** |
| `link-prefix`       | `/`                                   | Prefix for recursive links. This should be the path where you host Miasma, e.g. `/naughty-bots`.                                                                                                                                                                      |
| `link-count`        | `5`                                   | Number of recursive links to include in each response.                                                                                                                                                                                                                |
| `max-depth`         | `none`                                | Stop generating links once the scraper reaches the specified depth. This allows you to cut off scrapers after serving a desired amount of poison. _Use this in tandem with `link-count` to limit the number of active scrapers._                                      |
| `force-gzip`        | `false`                               | Always gzip responses regardless of the client's _Accept-Encoding_ header. **Forcing compression can help reduce egress costs.**                                                                                                                                      |
| `unsafe-allow-html` | `false`                               | Don't escape HTML characters in the poison source's responses. Escaping is enabled by default to prevent unintended client-side JavaScript execution in the event a real user stumbles upon a Miasma page.                                                            |
| `poison-source`     | `https://rnsaffn.com/poison2/?mask=0` | Proxy source for poisoned training data. Poison is embedded within Miasma's generated pages.                                                                                                                                                                          |
| `no-poison-cache`   | `false`                               | Disable poison source response caching.                                                                                                                                                                                                                               |

### Server Settings

| Option        | Default     | Description                                                                                                                                                       |
| ------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `port`        | `9999`      | The port Miasma will bind to.                                                                                                                                     |
| `host`        | `localhost` | The host address Miasma will bind to.                                                                                                                             |
| `unix-socket` |             | Bind to a [Unix domain socket](https://en.wikipedia.org/wiki/Unix_domain_socket){:target=blank} rather than a TCP address. _Only available on Unix-like systems._ |

### Metrics Settings

See the [metrics documentation](metrics#configuration).

## Tweaking Settings for Minimal Resource Usage

There are a few options that are particularly important for minimizing resource usage.

### Capping In-Flight Requests

`max-in-flight` directly constrains the resource usage of Miasma. This caps the number of in-flight concurrent requests that will be allowed at any given time. This is _not_ the same as the number of _concurrent_ requests.

For example, you might receive 100 requests per second from various scrapers that you redirect to your instance of Miasma, but you've set `max-in-flight` to `15`. At any given moment, 15 scrapers' request will be in-flight and consuming resources, while all other requests will receive a `429 Too Many Requests` response indicating they should try again after a delay.

### Response Compression

[Gzip compression](https://en.wikipedia.org/wiki/Gzip){:target=blank} is used to _greatly_ reduce the wire size of Miasma's responses. On average, gzipped responses `~70%` smaller than uncompressed responses.

By default, Miasma only compresses responses if the scraper includes an `Accept-Encoding: gzip` HTTP header. Enabling the `force-gzip` option instructs Miasma to always gzip responses, even in cases where a scraper may not be able to process that content. Depending on your traffic patterns and hosting environment, this can significantly reduce egress costs.

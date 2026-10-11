---
title: How to Trap Scrapers
icon: lucide/bot-off
---

# How to Trap Scrapers

Miasma does not handle bot detection on its own. You will need to set up a method of redirecting scrapers to your Miasma instance. There are a number of great open-source options for this, such as [anubis](https://github.com/techaroHQ/anubis){:target=blank}. Many hosting providers also have built-in methods for detecting bots. For this guide we'll follow a simpler approach - embedding hidden links directly into your site.

First, pick a location where you'll host Miasma. For this guide, we'll use `/naughty-bots`. We'll use [_Nginx_](https://nginx.org/) as our server's reverse proxy, but the same result can be achieved with a variety of setups.

When we're done, scrapers will be trapped like so:

<figure markdown="span">
![Scraper trapping flow](images/flow-chart-light.png#only-light){width="600"}
![Scraper trapping flow](images/flow-chart-dark.png#only-dark){width="600"}

<figcaption>Scraper trapping control flow</figcaption>
</figure>

## Embedding Hidden Links

Within our site, we'll include a few hidden links leading to `/naughty-bots`.

```html title="Hiding links in html"
<a
  href="/naughty-bots/"
  style="display: none;"
  aria-hidden="true"
  tabindex="-1"
>
  Amazing high quality data here!
</a>
```

The `style="display: none;"`, `aria-hidden="true"`, and `tabindex="-1"` attributes ensure links are fully invisible to human visitors, screen readers, and keyboard navigation. They will **only** be visible to scrapers.

## Configuring our Proxy

Since our hidden links point to `/naughty-bots/`, we'll configure this path to proxy requests to Miasma. Let's assume we're running Miasma on port `9855`.

```nginx title="nginx.conf"
http {
  server {
    location = /naughty-bots {
      port_in_redirect off;
      return 301 /naughty-bots/;
    }
    location /naughty-bots/ {
      # Proxy requests to Miasma
      proxy_pass http://localhost:9855/; # (1)!
    }
  }
}
```

1. :lucide-circle-alert: If you've bound Miasma to a Unix domain socket, use the following directive instead: `proxy_pass http://unix:/<path_to_socket>:/;`.

This configuration will catch all variations of the `/naughty-bots` path, e.g. `/naughty-bots`, `/naughty-bots/`, `/naughty-bots/12345`, etc.

## Run Miasma

Lastly, we'll start Miasma and specify `/naughty-bots` as our link prefix. This instructs Miasma to start links with `/naughty-bots/`, which ensures scrapers are properly routed through our proxy and back to Miasma.

Let's tune some of Miasma's [configuration settings](/settings).

We'll limit the number of max in-flight connections to 50. Any requests exceeding this limit will immediately receive a **429** response rather than being added to a queue. We'll also force Miasma to gzip compress all responses, and we'll use the link count and max depth options to release scrapers after they consume ~100K poisoned pages.

```sh
miasma --link-prefix '/naughty-bots' -p 9855 -c 50 --force-gzip --link-count 5 --max-depth 8
```

## Deploy

Let's deploy our site and watch as scrapers eat from our poison slop machine!

<figure markdown="span">
    ![Logs showing scrapers hitting Miasma](/images/logs.gif){width="900"}
</figure>

## Allowing Good Bots via `robots.txt`

Be sure to protect well-behaved bots and search engines from Miasma via your site's [`robots.txt`](https://developers.google.com/search/docs/crawling-indexing/robots/intro){:target=blank}.

```text title="robots.txt"
User-agent: *
Disallow: /naughty-bots
```

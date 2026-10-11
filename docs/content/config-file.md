---
title: Config File
icon: lucide/file-cog
---

# Using a Config File

You can specify Miasma's [configuration options](settings) by pointing to a configuration file.

```sh title="Specifying a config file"
miasma --config-file <path_to_config_file>
```

Miasma supports `YAML`, `TOML`, and `JSON` formats.

## Example Config Files

=== "YAML"

    ```yaml
    server:
      host: 127.0.0.1
      port: 9999
      unix_socket: /tmp/miasma.sock # (1)!

    max_in_flight: 20
    link_prefix: /bots
    link_count: 5
    max_depth: 10
    force_gzip: true
    unsafe_allow_html: false
    poison_source: https://example.com
    disable_poison_cache: false

    metrics:
      db_path: miasma.db
      username: admin
      password: change-me
      endpoint: /request-metrics
    ```

    1. :lucide-info: Only available on Unix-like systems.

=== "TOML"

    ```toml
    max_in_flight = 20
    link_prefix = "/bots"
    link_count = 5
    max_depth = 10
    force_gzip = true
    unsafe_allow_html = false
    poison_source = "https://example.com"
    disable_poison_cache = false

    [server]
    host = "127.0.0.1"
    port = 9999
    unix_socket = "/tmp/miasma.sock" # (1)!

    [metrics]
    db_path = "miasma.db"
    username = "admin"
    password = "change-me"
    endpoint = "/request-metrics"
    ```

    1. :lucide-info: Only available on Unix-like systems.

=== "JSON"

    ```json
    {
      "server": {
        "host": "127.0.0.1",
        "port": 9999,
        "unix_socket": "/tmp/miasma.sock" // (1)!
      },
      "max_in_flight": 20,
      "link_prefix": "/bots",
      "link_count": 5,
      "max_depth": 10,
      "force_gzip": true,
      "unsafe_allow_html": false,
      "poison_source": "https://example.com",
      "disable_poison_cache": false,
      "metrics": {
        "db_path": "miasma.db",
        "username": "admin",
        "password": "change-me",
        "endpoint": "/request-metrics"
      }
    }
    ```

    1. :lucide-info: Only available on Unix-like systems.

## Overriding Values

All fields are optional and any settings can be overridden by providing the corresponding flag.

For example, if your config file looks like this...

=== "YAML"

    ```yaml
    server:
      port: 9999
    ```

=== "TOML"

    ```toml
    [server]
    port = 9999
    ```

=== "JSON"

    ```json
    {
      "server": {
        "port": 9999
      }
    }
    ```

...but you would like to override the `port` value, simply provide the flag.

```sh title="Overriding a config file value"
miasma -f <path_to_config_file> --port 8000
```

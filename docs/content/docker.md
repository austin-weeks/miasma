---
title: Docker
icon: fontawesome/brands/docker
---

# Running with Docker

Miasma offers an official [docker image](https://hub.docker.com/r/austinweeks/miasma){:target=blank}.

Create a container using your preferred docker setup. Miasma binds to port `9999`, but you can [expose whatever port you'd like](https://docs.docker.com/get-started/docker-concepts/running-containers/publishing-ports/){:target=blank}.

=== "Local"

    ```sh
    docker run --rm -p 9999:9999 austinweeks/miasma:latest
    ```

=== "Docker Compose"

    ```yaml
    services:
      miasma:
        image: austinweeks/miasma:latest
        ports:
          - 9999:9999
    ```

<!-- TODO: add setup example for kubernetes -->

You can view a response with:

```sh
curl --compressed http://localhost:9999
```

## Configuration

You can pass [configuration options](settings) as command-line flags:

=== "Local"

    ```sh
    docker run --rm -p 9999:9999 austinweeks/miasma:latest \
      --link-prefix '/bots' \
      --max-in-flight 10 \
      --force-gzip
    ```

=== "Docker Compose"

    ```yaml
    services:
      miasma:
        image: austinweeks/miasma:latest
        command: >
          --link-prefix /bots
          --max-in-flight 30
          --force-gzip
        ports:
          - 9999:9999
    ```

You may also create a [config file](config-file) and provide access to the container.

=== "Local"

    ```sh
    docker run --rm -p 9999:9999 \
      -v <PATH_TO_CONFIG>:/etc/miasma-config.yaml:ro \
      austinweeks/miasma:latest \
      --config-file /etc/miasma-config.yaml # (1)!
    ```

    1. :lucide-info: This gives the container read-only access to the config file on your machine. Note that the path must be absolute. You can easily grab this by using `$(pwd)/config.yaml`.

=== "Docker Compose"

    ```yaml
    services:
      miasma:
        image: austinweeks/miasma:latest
        volumes:
          - <DIR_WITH_CONFIG>:/etc/config-dir:ro # (1)!
        command: --config-file /etc/config-dir/miasma-config.yaml
        ports:
          - 9999:9999
    ```

    1. :lucide-info: This gives the container read-only access to a directory with the config file on your machine. The path may be relative or absolute. **Note**: You must bind to a directory, as Docker Compose does not allow binding to specific files.

## Tracking Metrics

Miasma's [metrics feature](metrics) allows you to track scrapers that interact with Miasma. Because the feature writes data to a [SQLite](https://en.wikipedia.org/wiki/SQLite){:target="_blank"} file, it's necessary to provide the container with a volume where this data can be stored.

=== "Local"

    ```sh
    docker run --rm -p 9999:9999 \
      -v <PATH_TO_DATA_DIR>:/data \
      austinweeks/miasma:latest \
      --metrics-db-path /data/miasma.db \
      --metrics-username <USERNAME> \
      --metrics-password <PASSWORD> # (1)!
    ```

    1. :lucide-info: This gives the container write access to a directory of your choosing. Miasma will create a `miasma.db` file in this directory on startup, or reuse one from previous runs.

=== "Docker Compose"

    ```yaml
    services:
      miasma:
        image: austinweeks/miasma:latest
        ports:
          - 9999:9999
        volumes:
          - miasma-metrics:/data
        command: >
          --metrics-db-path /data/miasma.db
          --metrics-username <USERNAME>
          --metrics-password <PASSWORD>

    volumes:
      miasma-metrics:
    ```

Please see the [metrics page](/metrics) for more information on setting up metrics.

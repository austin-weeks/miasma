help:
    @just --list

local-ci:
    cargo build
    cargo fmt --check
    cargo clippy --no-deps --all-targets -- -D warnings
    cargo test
    cargo publish --dry-run --allow-dirty
    just docker-test

docker-build *args:
    docker build -t miasma:local-dev . {{ args }}

docker-run *args:
    docker run --rm -p 9999:9999 miasma:local-dev {{ args }}

docker-test:
    #!/usr/bin/env bash
    set -euo pipefail

    just docker-build

    container_id=$(docker run -d --rm \
        -p 9999:9999 \
        miasma:local-dev \
        --poison-source https://example.com \
    )

    trap 'docker rm -f "$container_id"' EXIT

    sleep 5
    curl -fsS http://localhost:9999 -o /dev/null

    echo OK

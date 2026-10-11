---
icon: lucide/pc-case
---

# Running Locally

Miasma is easily run as a standalone executable that can be integrated into your server environment in whatever way you choose.

## Installation Options

Install with your favorite package manager:

[![Package Managers](https://repology.org/badge/vertical-allrepos/miasma.svg?exclude_unsupported=1)](https://repology.org/project/miasma/versions){:target=blank}

Build from source with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html){:target=blank}:

```sh
cargo install miasma
```

Download a pre-built binary from [our GitHub releases](https://github.com/austin-weeks/miasma/releases){:target=blank}.

## Usage

Run Miasma with default settings:

```sh
miasma
```

By default, Miasma binds to port `9999` on `localhost`. View a response with:

```sh
curl --compressed http://localhost:9999
```

View all available [configuration](settings) flags:

```sh
miasma --help
```

Once running, you can stop the server with ++ctrl+c++.

---
title: Library
icon: lucide/code-xml
---

# Using Miasma in an Existing Axum Server

If you happen to have an existing Rust server using the [axum crate](https://docs.rs/axum/latest/axum/){:target="_blank"}, you may find it easier to use Miasma's library API directly rather than spinning it up as a separate service.

```sh title="Adding Miasma as a dependency"
cargo add miasma
```

Define Miasma's [configuration options](/settings), build an axum router, then nest it within your server's routes.

```Rust title="Using Miasma's Axum Router"
use miasma::MiasmaConfig;
use axum::{Router, routing::get};
use tokio::net::TcpListener;

// Set up Miasma's axum router
let config = MiasmaConfig::builder()
    .link_prefix("/bots")
    .build();
let miasma_router = miasma::new_miasma_router(config).unwrap();

// Nest the routes within your application
let my_router = Router::new()
    .route("/", get(|| async { "ok" }))
    .nest("/bots", miasma_router);

// Start the server
let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
axum::serve(listener, my_router).await.unwrap();
```

See Miasma's [docs.rs page](https://docs.rs/miasma){:target=blank} for more information.

!!! info "Licensing Considerations"

    Miasma is licensed under GPL-v3, a strong copyleft license requiring derivative works be released as free software under the same license.

    Consuming Miasma as a library does not require you to release your source code. However, if you distribute an application that incorporates Miasma's library, the GPL-v3 _may_ require the combined work to be licensed under GPL-v3 and that the accompanying source code be released as free software.

# syntax=docker/dockerfile:1

FROM rust:1.86-slim AS builder
WORKDIR /usr/src/miasma

# Copy manifest and fetch dependencies first for better caching.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() { println!("cargo build placeholder"); }' > src/main.rs
RUN cargo fetch --locked

# Copy the full source and build the release binary.
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/miasma/target/release/miasma /usr/local/bin/miasma
EXPOSE 9999
ENTRYPOINT ["/usr/local/bin/miasma"]
CMD ["--host", "0.0.0.0"]

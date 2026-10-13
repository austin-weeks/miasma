# --- BUILD ---
FROM rust:bookworm AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock .
COPY crates/ crates/

# Build just dependencies so they can be cached between image builds
RUN mkdir src && echo "fn main() {}" > src/main.rs && touch src/lib.rs
RUN cargo build --release

# Build actual miasma binary
COPY src/ src/
# COPY preserves timestamps; touch invalidates Cargo's cached dummy artifacts.
RUN touch src/main.rs src/lib.rs && cargo build --release

# --- RUNTIME ---
FROM gcr.io/distroless/cc-debian12

# Copy Sqlite Binary
COPY --from=builder /usr/lib/*-linux-gnu/libsqlite3.so* /usr/lib/

COPY --from=builder /build/target/release/miasma /usr/local/bin/miasma

# make app reachable outside the container
ENV MIASMA_HOST=0.0.0.0

EXPOSE 9999

ENTRYPOINT ["miasma"]

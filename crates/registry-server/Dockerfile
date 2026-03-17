# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /app

# Cache dependency compilation separately from source changes.
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

RUN cargo build --release -p tsx-registry

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tsx-registry /usr/local/bin/tsx-registry

ENV PORT=8080
ENV DATA_DIR=/data

VOLUME ["/data"]

EXPOSE 8080

CMD ["tsx-registry"]

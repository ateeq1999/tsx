# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.88-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependency compilation separately from source changes.
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY migrations/ migrations/

RUN cargo build --release -p tsx-registry

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tsx-registry /usr/local/bin/tsx-registry

ENV PORT=8282
ENV DATA_DIR=/data

VOLUME ["/data"]

EXPOSE 8282

CMD ["tsx-registry"]

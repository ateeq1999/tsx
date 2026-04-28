# Multi-stage build for tsx-registry (Railway deployment)
# Uses cargo-chef to cache dependency compilation separately from source changes.
#
# ── Stage 1: Chef ─────────────────────────────────────────────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# ── Stage 2: Planner ──────────────────────────────────────────────────────────
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: Builder ──────────────────────────────────────────────────────────
FROM chef AS builder

# Install build dependencies for native-tls (SQLx)
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 1. Cook dependencies only (cached unless Cargo.toml / Cargo.lock changes)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release -p tsx-registry --recipe-path recipe.json

# 2. Build the application (invalidated only when source changes)
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
ENV SQLX_OFFLINE=true
RUN cargo build --release -p tsx-registry

# ── Stage 4: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tsx-registry /usr/local/bin/tsx-registry

# Railway injects PORT at runtime; default to 8282 for local testing.
ENV PORT=8282
ENV DATA_DIR=/app/data

EXPOSE 8282

CMD ["tsx-registry"]

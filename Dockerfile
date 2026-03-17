# Build the tsx-registry server from the workspace root.
# Railway deploys this via: docker build -f Dockerfile .
#
# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests and lockfile first (layer-cache friendly)
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# SQLx offline mode — uses the pre-generated query cache (.sqlx/)
# so no live database connection is needed at build time.
ENV SQLX_OFFLINE=true

RUN cargo build --release -p tsx-registry

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tsx-registry /usr/local/bin/tsx-registry

# Railway injects PORT at runtime; default to 8080 for local testing.
ENV PORT=8080
ENV DATA_DIR=/data

EXPOSE 8080

CMD ["tsx-registry"]

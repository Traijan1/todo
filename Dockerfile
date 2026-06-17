# syntax=docker/dockerfile:1
FROM rust:1.89.0-slim AS builder

WORKDIR /usr/src/

# ── System deps (cached unless changed) ──────────────────────────────────────
RUN apt-get update && apt-get install -y \
    curl ca-certificates pkg-config libssl-dev libsqlite3-dev \
    && curl -fsSL https://deb.nodesource.com/setup_current.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# ── Node deps (cached unless package.json changes) ────────────────────────────
COPY frontend/package.json frontend/package-lock.json* frontend/
RUN cd frontend && npm install --legacy-peer-deps

# ── Rust dep cache (cached unless Cargo.lock changes) ─────────────────────────
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml migration/
RUN mkdir -p src/bin migration/src \
    && echo 'fn main(){}' > src/bin/main.rs \
    && echo '' > src/lib.rs \
    && echo '' > migration/src/lib.rs \
    && cargo build --release 2>/dev/null || true \
    && rm -rf src migration/src

# ── Full build (only app code recompiles on source changes) ───────────────────
COPY . .
RUN touch src/bin/main.rs migration/src/lib.rs
RUN cd frontend && npm run build
RUN cargo build --release

# ── Runtime image ─────────────────────────────────────────────────────────────
FROM debian:trixie-slim

WORKDIR /usr/app

RUN apt-get update && apt-get install -y libsqlite3-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /usr/app/data

COPY --from=builder /usr/src/frontend/dist frontend/dist
COPY --from=builder /usr/src/config config
COPY --from=builder /usr/src/target/release/todo-cli todo-cli

ENTRYPOINT ["/usr/app/todo-cli"]

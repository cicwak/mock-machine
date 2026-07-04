# syntax=docker/dockerfile:1.7

FROM rust:1.96.0-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY apps/backend/Cargo.toml apps/backend/Cargo.toml
COPY apps/backend/src apps/backend/src
COPY apps/backend/migrations apps/backend/migrations
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS deps
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release -p mock-machine-backend --recipe-path recipe.json

FROM chef AS builder
COPY Cargo.toml Cargo.lock ./
COPY apps/backend/Cargo.toml apps/backend/Cargo.toml
COPY apps/backend/src apps/backend/src
COPY apps/backend/migrations apps/backend/migrations
COPY --from=deps /app/target /app/target
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release -p mock-machine-backend \
    && cp /app/target/release/mock-machine /usr/local/bin/mock-machine

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/mock-machine /usr/local/bin/mock-machine

EXPOSE 8080
CMD ["mock-machine"]

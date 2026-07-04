FROM rust:1.88-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY apps/backend/Cargo.toml apps/backend/Cargo.toml
COPY apps/backend/src apps/backend/src

RUN cargo build --release -p mock-machine-backend

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mock-machine /usr/local/bin/mock-machine

EXPOSE 8080
CMD ["mock-machine"]

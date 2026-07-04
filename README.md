# Mock Machine

Mock Machine is a self-hosted HTTP mock service with an admin web panel. It captures unknown HTTP requests, lets you convert them into reusable mock routes, and stores response scenarios, request samples, and binary assets locally.

The default stack runs behind nginx and includes:

- Rust/Axum backend.
- React/TypeScript/MUI admin panel.
- PostgreSQL persistence.
- Redis cache and realtime fan-out.
- MinIO object storage.
- Docker Compose for local and small-team deployments.

## Status

Mock Machine is in early `0.x` development. The project is usable for local development and internal environments, but APIs, database schema, and configuration may still change before `1.0.0`.

## Quick Start

Requirements:

- Docker
- Docker Compose

Run:

```sh
git clone https://github.com/cicwak/mock-machine.git
cd mock-machine
cp .env.example .env
docker compose up --build
```

Open:

- Admin panel: `http://localhost:8088/mockadmin`
- Backend health check: `http://localhost:8088/mockadminapi/health`
- MinIO console: `http://localhost:9001`

The default credentials in `.env.example` are for local development only. Change them before running Mock Machine in a shared or network-accessible environment.

## Common Flow

Send a request to a route that has not been configured:

```sh
curl -i http://localhost:8088/api/not-configured
```

Inspect captured unknown requests:

```sh
curl http://localhost:8088/mockadminapi/unknown-requests
```

Convert an unknown request into a mock route:

```sh
curl -X POST http://localhost:8088/mockadminapi/unknown-requests/{id}/convert \
  -H 'content-type: application/json' \
  -d '{"scenario":{"status_code":200,"response_body":"{\"ok\":true}","response_headers":{"content-type":"application/json"}}}'
```

Call the same public route again:

```sh
curl -i http://localhost:8088/api/not-configured
```

## Repository Layout

- `apps/backend` - Rust/Axum backend service.
- `apps/frontend` - React/TypeScript/MUI admin panel.
- `infra/nginx` - reverse proxy config for `/mockadmin`, `/mockadminapi`, Socket.IO, and public mock routes.
- `infra/postgres/init` - first-run PostgreSQL schema and seed scripts.
- `infra/minio` - MinIO bucket, private ACL, and app read/write user bootstrap.
- `docs` - product requirements, ADRs, and operator documentation.

## Configuration

Copy `.env.example` to `.env` and adjust values for your environment. The most important settings are:

- `NGINX_PORT` - external HTTP port. Defaults to `8088`.
- `APP_STORAGE` - `postgres` by default. Set to `in_memory` to store mock routes and captured unknown requests in Redis.
- `DATABASE_URL` - required when `APP_STORAGE=postgres`.
- `REDIS_URL` - required when `APP_STORAGE=in_memory`; also used for Socket.IO pub/sub when configured.
- `S3_ENDPOINT`, `S3_BUCKET`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` - object storage settings.
- `RUST_LOG` - backend log filtering.

See [Configuration](docs/configuration.md) for details.

## Useful Endpoints

- `GET /mockadminapi/health` - backend health check.
- `GET /mockadminapi/unknown-requests` - list captured unconfigured requests.
- `GET /mockadminapi/unknown-requests/{id}` - inspect the last captured request sample.
- `POST /mockadminapi/unknown-requests/{id}/convert` - create a mock route and active response scenario from an unknown request.
- `GET /mockadminapi/routes` - list configured mock routes.
- `PUT /mockadminapi/assets/{key}` - store a binary asset through MinIO when S3 settings are configured.
- `GET /mockadminapi/assets/{key}` - read a binary asset through MinIO when S3 settings are configured.

See [API Notes](docs/api.md) for examples.

## Documentation

- [Installation](docs/installation.md)
- [Configuration](docs/configuration.md)
- [Storage](docs/storage.md)
- [Backup and Restore](docs/backup-restore.md)
- [Upgrading](docs/upgrading.md)
- [API Notes](docs/api.md)
- [Changelog](CHANGELOG.md)
- [Security Policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## Development

Backend:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Frontend:

```sh
cd apps/frontend
npm ci
npm run build
```

Full stack:

```sh
cp .env.example .env
docker compose up --build
```

## Release Process

The project uses semantic versioning.

1. Update Rust workspace and frontend package versions.
2. Update `CHANGELOG.md`.
3. Run backend, frontend, and Docker checks.
4. Create and push a tag, for example `v0.1.0`.
5. Publish a GitHub Release with installation notes, upgrade notes, and known limitations.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

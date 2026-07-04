# Mock Machine

Mock Machine is a local HTTP mock service with an admin web panel, PostgreSQL persistence, Redis cache, MinIO object storage and nginx as the single external entrypoint.

## Repository layout

- `apps/backend` - Rust/Axum backend service.
- `apps/frontend` - React/TypeScript/MUI admin panel.
- `infra/nginx` - reverse proxy config for `/mockadmin`, `/mockadminapi` and public mock routes.
- `infra/postgres/init` - first-run PostgreSQL schema and seed scripts.
- `infra/minio` - MinIO bucket, private ACL and app read/write user bootstrap.
- `docs` - product requirements and ADRs.

## Local run

```sh
cp .env.example .env
docker compose up --build
```

Open the admin panel at `http://localhost:8088/mockadmin`.

Useful endpoints:

- `GET http://localhost:8088/mockadminapi/health` - backend health check.
- `http://localhost:9001` - MinIO console.

Default local MinIO credentials are defined in `.env.example`. The `minio-bootstrap` service creates:

- private bucket from `MINIO_BUCKET`;
- app user from `MINIO_ACCESS_KEY` and `MINIO_SECRET_KEY`;
- `mock-machine-readwrite` policy scoped to that bucket.

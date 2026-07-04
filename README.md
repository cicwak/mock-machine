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

Set `APP_STORAGE=in_memory` to store mock routes and captured unknown requests in Redis instead of PostgreSQL. In this mode `REDIS_URL` is required.

Open the admin panel at `http://localhost:8088/mockadmin`.

The admin panel connects to Socket.IO at `/socket.io/`. When `REDIS_URL` is configured, the backend uses Socketioxide's Redis pub/sub adapter so realtime unknown-request broadcasts reach clients connected to other backend instances. The adapter requires Redis 7+ with RESP3; the backend appends `protocol=resp3` to `REDIS_URL` for Socket.IO when it is not already present.

Useful endpoints:

- `GET http://localhost:8088/mockadminapi/health` - backend health check.
- `GET http://localhost:8088/mockadminapi/unknown-requests` - list captured unconfigured requests.
- `GET http://localhost:8088/mockadminapi/unknown-requests/{id}` - inspect the last captured request sample.
- `POST http://localhost:8088/mockadminapi/unknown-requests/{id}/convert` - create a mock route and active response scenario from an unknown request.
- `GET http://localhost:8088/mockadminapi/routes` - list configured mock routes.
- `PUT/GET http://localhost:8088/mockadminapi/assets/{key}` - store and read binary assets through MinIO when S3 settings are configured.
- `http://localhost:9001` - MinIO console.

Unknown request flow:

```sh
curl -i http://localhost:8088/api/not-configured
curl http://localhost:8088/mockadminapi/unknown-requests
curl -X POST http://localhost:8088/mockadminapi/unknown-requests/{id}/convert \
  -H 'content-type: application/json' \
  -d '{"scenario":{"status_code":200,"response_body":"{\"ok\":true}","response_headers":{"content-type":"application/json"}}}'
curl -i http://localhost:8088/api/not-configured
```

Default local MinIO credentials are defined in `.env.example`. The `minio-bootstrap` service creates:

- private bucket from `MINIO_BUCKET`;
- app user from `MINIO_ACCESS_KEY` and `MINIO_SECRET_KEY`;
- `mock-machine-readwrite` policy scoped to that bucket.

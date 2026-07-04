# Storage

Mock Machine uses PostgreSQL, Redis, and S3-compatible object storage in the default Docker Compose stack.

## PostgreSQL

PostgreSQL is the default source of truth when `APP_STORAGE=postgres`.

It stores:

- mock routes;
- response scenarios;
- profiles;
- captured unknown requests;
- object asset metadata.

The Compose service persists data in the `postgres-data` Docker volume.

## Redis

Redis is used for:

- `APP_STORAGE=in_memory` mode;
- Socket.IO pub/sub fan-out when multiple backend instances are connected to Redis.

The Socket.IO Redis adapter requires Redis 7+ with RESP3. When needed, the backend appends `protocol=resp3` to `REDIS_URL` for Socket.IO if it is not already present.

The Compose service persists Redis data in the `redis-data` Docker volume.

## MinIO

MinIO provides S3-compatible storage for binary assets.

The `minio-bootstrap` service creates:

- the private bucket from `MINIO_BUCKET`;
- the app user from `MINIO_ACCESS_KEY` and `MINIO_SECRET_KEY`;
- a bucket-scoped `mock-machine-readwrite` policy.

The Compose service persists object data in the `minio-data` Docker volume.

## Local Development Defaults

Values in `.env.example` are safe for local repeatable setup, but they are not production secrets. Replace them before sharing the stack on a network.

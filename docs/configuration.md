# Configuration

Configuration is provided through environment variables. For Docker Compose, copy `.env.example` to `.env` and edit the values there.

## Ports

| Variable | Default | Description |
| --- | --- | --- |
| `NGINX_PORT` | `8088` | External HTTP port exposed by nginx. |
| `BACKEND_PORT` | `8080` | Backend port used for direct local development. |
| `FRONTEND_PORT` | `5173` | Frontend dev server port used for direct local development. |

The Docker Compose entrypoint is nginx. Most users should access Mock Machine through `NGINX_PORT`.

## Backend

| Variable | Default | Description |
| --- | --- | --- |
| `APP_STORAGE` | `postgres` | Storage mode. Use `postgres` or `in_memory`. |
| `BIND_ADDR` | `0.0.0.0:8080` | Backend bind address inside the container. |
| `DATABASE_URL` | Compose PostgreSQL URL | Required when `APP_STORAGE=postgres`. |
| `REDIS_URL` | Compose Redis URL | Required when `APP_STORAGE=in_memory`; also used for realtime fan-out when configured. |
| `RUST_LOG` | `mock_machine=info,tower_http=info` | Backend log filter. |

## Object Storage

| Variable | Default | Description |
| --- | --- | --- |
| `S3_ENDPOINT` | `http://minio:9000` | S3-compatible endpoint. |
| `S3_REGION` | `us-east-1` | S3 region. |
| `S3_BUCKET` | `mock-machine-files` | Bucket for binary assets. |
| `AWS_ACCESS_KEY_ID` | Derived from `MINIO_ACCESS_KEY` in Compose | App access key. |
| `AWS_SECRET_ACCESS_KEY` | Derived from `MINIO_SECRET_KEY` in Compose | App secret key. |

When S3 settings are missing, binary asset endpoints may not be available.

## MinIO Bootstrap

| Variable | Default | Description |
| --- | --- | --- |
| `MINIO_ROOT_USER` | `mock_machine_root` | MinIO root user for bootstrap and console access. |
| `MINIO_ROOT_PASSWORD` | `mock_machine_root_password` | MinIO root password. |
| `MINIO_BUCKET` | `mock-machine-files` | Bucket created by bootstrap. |
| `MINIO_REGION` | `us-east-1` | MinIO region. |
| `MINIO_ACCESS_KEY` | `mock_machine_app` | Application user created by bootstrap. |
| `MINIO_SECRET_KEY` | `mock_machine_app_secret` | Application user secret. |

## Frontend

| Variable | Default | Description |
| --- | --- | --- |
| `VITE_API_BASE_URL` | `/mockadminapi` in Compose | Admin API base path. |
| `VITE_SOCKET_IO_URL` | empty | Optional Socket.IO origin override. |

The admin panel normally derives Socket.IO origin from `VITE_API_BASE_URL` in development and from the current origin behind nginx.

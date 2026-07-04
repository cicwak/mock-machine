# Backup and Restore

Back up PostgreSQL and MinIO together. They represent different parts of the same application state.

## PostgreSQL Backup

Create a database dump:

```sh
docker compose exec postgres pg_dump \
  -U "${POSTGRES_USER:-mock_machine}" \
  -d "${POSTGRES_DB:-mock_machine}" \
  > mock-machine-postgres.dump
```

Restore into a running PostgreSQL container:

```sh
cat mock-machine-postgres.dump | docker compose exec -T postgres psql \
  -U "${POSTGRES_USER:-mock_machine}" \
  -d "${POSTGRES_DB:-mock_machine}"
```

For a clean restore, start from an empty database or recreate the `postgres-data` volume first.

## MinIO Backup

For local Docker volume backup, stop writes first and archive the volume data with your preferred Docker volume backup tooling.

For S3-compatible backup, use `mc mirror` or another S3 client to copy the configured bucket:

```sh
mc mirror local/mock-machine-files ./mock-machine-files-backup
```

Configure the `local` alias with your MinIO endpoint and credentials before running the command.

## Redis Backup

Redis is only the primary route store when `APP_STORAGE=in_memory`. In the default PostgreSQL mode, Redis data is cache and realtime infrastructure.

If you run with `APP_STORAGE=in_memory`, back up the `redis-data` volume or Redis persistence files before upgrading or moving the stack.

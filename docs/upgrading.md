# Upgrading

Mock Machine is in `0.x` development, so releases may include breaking changes.

## Recommended Upgrade Flow

1. Read `CHANGELOG.md` for the target version.
2. Back up PostgreSQL and MinIO.
3. Pull the new code or image versions.
4. Review `.env.example` for new or changed variables.
5. Apply your environment-specific `.env` changes.
6. Start the stack.
7. Check `GET /mockadminapi/health`.
8. Verify the admin panel and a representative mock route.

## Docker Compose Source Checkout

```sh
git fetch --tags
git checkout v0.1.0
docker compose up --build -d
```

Replace `v0.1.0` with the target release tag.

## Rollback

1. Stop the stack.
2. Restore the previous code or image versions.
3. Restore PostgreSQL and MinIO backups if the failed upgrade changed stored data.
4. Start the stack and verify the health endpoint.

Because database migrations may be one-way during `0.x`, take backups before every upgrade.

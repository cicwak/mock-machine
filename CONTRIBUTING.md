# Contributing

Thanks for taking the time to improve Mock Machine.

## Development Setup

Requirements:

- Docker and Docker Compose.
- Rust toolchain with the edition used by the workspace.
- Node.js and npm for the frontend.

Run the full stack:

```sh
cp .env.example .env
docker compose up --build
```

Run backend checks:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run frontend checks:

```sh
cd apps/frontend
npm ci
npm run build
```

## Pull Requests

- Keep changes focused.
- Include tests when changing behavior with meaningful risk.
- Update docs when changing setup, configuration, API behavior, or release process.
- Keep generated build output out of commits.
- Prefer small PRs over broad rewrites.

## Commit Style

Use short imperative commit messages, for example:

```text
add release documentation
fix unknown request conversion
update frontend build config
```

## License

By contributing to this project, you agree that your contribution is licensed under the Apache License, Version 2.0.

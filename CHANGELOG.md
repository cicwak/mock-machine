# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Release documentation for installation, configuration, storage, backup, upgrades, API usage, security, and contribution workflow.
- GitHub Actions CI for backend, frontend, and Docker build validation.
- Apache-2.0 project license.

## [0.1.0] - 2026-07-04

### Added

- Docker Compose stack with nginx, backend, frontend, PostgreSQL, Redis, MinIO, and MinIO bootstrap.
- Rust/Axum backend service.
- React/TypeScript/MUI admin panel.
- Unknown request capture flow.
- Conversion from captured unknown requests into configured mock routes.
- PostgreSQL-backed route, scenario, profile, unknown-request, and object-asset persistence.
- Redis-backed in-memory mode for development and lightweight usage.
- Socket.IO realtime broadcasts for unknown request updates.
- MinIO-backed binary asset storage.
- Product requirements and architecture decision records.

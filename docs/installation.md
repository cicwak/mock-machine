# Installation

Mock Machine is distributed as a Docker Compose stack.

## Requirements

- Docker
- Docker Compose

## Local Installation

```sh
git clone https://github.com/cicwak/mock-machine.git
cd mock-machine
cp .env.example .env
docker compose up --build
```

Open `http://localhost:8088/mockadmin`.

## Running in the Background

```sh
docker compose up --build -d
```

View logs:

```sh
docker compose logs -f
```

Stop the stack:

```sh
docker compose down
```

Stop and remove persisted local data:

```sh
docker compose down -v
```

## Production Notes

Mock Machine is not hardened by default for public internet exposure.

Before running it in a shared environment:

- change every value copied from `.env.example`;
- restrict network access to trusted users;
- put the admin panel behind authentication;
- back up PostgreSQL and MinIO data;
- review logs and captured request data for sensitive values.

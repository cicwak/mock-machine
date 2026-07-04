# ADR 0002: Routing and nginx

## Status

Accepted

## Context

Система должна иметь один внешний порт, но разные внутренние сервисы для админ-панели и mock backend. Пользователь должен открывать панель и дергать mock endpoints через один host.

## Decision

Использовать nginx как единственную внешнюю точку входа.

Правила маршрутизации:

- `/mockadmin` проксируется во frontend;
- `/mockadmin/*` проксируется во frontend;
- `/mockadminapi` проксируется в backend administrative API;
- `/mockadminapi/*` проксируется в backend administrative API;
- все остальные пути проксируются в backend mock service.

Административное API backend должно иметь обязательный отдельный префикс `/mockadminapi` и должно быть доступно frontend через nginx.

## Rationale

Такой подход позволяет:

- держать frontend и backend на разных внутренних портах;
- отдавать пользователю один внешний URL;
- не смешивать frontend routing и mock routing;
- позднее добавить TLS, rate limiting или auth на уровне nginx.

## Consequences

- `/mockadmin` и `/mockadmin/*` зарезервированы и не могут использоваться как mock routes.
- `/mockadminapi` и `/mockadminapi/*` зарезервированы и не могут использоваться как mock routes.
- Backend должен корректно обрабатывать все остальные пути как потенциальные mock requests или admin API.
- Frontend должен быть собран с учетом base path `/mockadmin`.

## Project subdomains

Для изоляции проектов в production mock-запросы могут идти на wildcard
subdomain вида `<project-key>.mock-machine.example.com`. Nginx извлекает
`project-key` из hostname и передает его backend-у в заголовке
`X-Mock-Project`.

Примеры конфигурации лежат в
`infra/nginx/project-subdomains.example.conf`.

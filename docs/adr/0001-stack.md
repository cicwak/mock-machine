# ADR 0001: Technology Stack

## Status

Accepted

## Context

Mock Machine должен включать web-панель, backend mock service, базу данных, кеш и объектное хранилище. Система ориентирована на локальный запуск через Docker Compose и должна оставаться достаточно простой для самостоятельной разработки.

## Decision

Использовать следующий стек:

- Frontend: React + TypeScript.
- UI kit: MUI.
- Backend: Rust.
- Web framework: Axum.
- Database: PostgreSQL.
- ORM: SeaORM.
- Redis client: `redis-rs`.
- Object storage: MinIO через S3-compatible API.

## Rationale

React + TypeScript дают предсказуемую основу для интерактивной админ-панели с деревом маршрутов, формами сценариев и unknown request inbox.

MUI подходит для административного интерфейса: таблицы, дерево, формы, tabs, dialogs и feedback-компоненты доступны из коробки.

Rust + Axum подходят для backend service, который должен принимать произвольные HTTP-запросы, быстро матчить маршруты и отдавать ответы с минимальными накладными расходами.

PostgreSQL является основным надежным хранилищем конфигураций и unknown requests.

SeaORM выбран как async ORM для Rust, совместимый с PostgreSQL и миграциями.

Redis используется для быстрого чтения активных маршрутов и сценариев в runtime path.

MinIO добавляется в базовую инфраструктуру как подготовка к мокированию файлов и бинарных ответов.

## Consequences

- Frontend и backend разворачиваются отдельными сервисами.
- nginx объединяет их под одним внешним портом.
- Backend должен иметь четкое разделение admin API и public mock handler.
- Схема данных должна проектироваться с учетом PostgreSQL как source of truth и Redis как cache.

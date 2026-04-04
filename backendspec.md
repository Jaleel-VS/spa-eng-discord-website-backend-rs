# Hablemos Website Backend — Rust Rewrite Spec

> Rust rewrite of [spa-eng-discord-website-backend](https://github.com/Jaleel-VS/spa-eng-discord-website-backend) (Go).
> Frontend: [the-spanish-english-discord-server](https://github.com/Jaleel-VS/the-spanish-english-discord-server) (React + TanStack Router, deployed on Vercel).

## Overview

REST API for the Spanish-English Discord community website. Two domains: **podcasts** (language-learning resource directory) and **link reports** (dead-link flagging by users). PostgreSQL database, deployed on Railway via Docker.

---

## Database Schema

### `podcasts`

| Column       | Type                     | Constraints                          |
|--------------|--------------------------|--------------------------------------|
| `id`         | `UUID`                   | PK, `DEFAULT gen_random_uuid()`      |
| `title`      | `VARCHAR(255)`           | NOT NULL                             |
| `description`| `VARCHAR(1000)`          | NOT NULL                             |
| `image_url`  | `TEXT`                   | NOT NULL                             |
| `language`   | `VARCHAR(10)`            | NOT NULL, CHECK (`en`, `es`, `both`) |
| `level`      | `VARCHAR(20)`            | NOT NULL, CHECK (`beginner`, `intermediate`, `advanced`) |
| `country`    | `VARCHAR(100)`           | NOT NULL                             |
| `topic`      | `VARCHAR(100)`           | NOT NULL                             |
| `url`        | `TEXT`                   | NOT NULL                             |
| `archived`   | `BOOLEAN`                | NOT NULL, DEFAULT `false`            |
| `created_at` | `TIMESTAMPTZ`            | NOT NULL, DEFAULT `NOW()`            |
| `updated_at` | `TIMESTAMPTZ`            | NOT NULL, DEFAULT `NOW()`            |

Indexes:
- `idx_podcasts_language` on `language`
- `idx_podcasts_level` on `level`
- `idx_podcasts_archived` on `archived`

### `link_reports`

| Column        | Type           | Constraints                          |
|---------------|----------------|--------------------------------------|
| `id`          | `UUID`         | PK, `DEFAULT gen_random_uuid()`      |
| `podcast_id`  | `UUID`         | NOT NULL, FK → `podcasts(id)` ON DELETE CASCADE |
| `reporter_ip` | `INET`         | NULLABLE                             |
| `created_at`  | `TIMESTAMPTZ`  | NOT NULL, DEFAULT `NOW()`            |

Constraints:
- `UNIQUE (podcast_id, reporter_ip) WHERE reporter_ip IS NOT NULL` — one report per IP per podcast (deduplication)

Indexes:
- `idx_link_reports_podcast_id` on `podcast_id`

---

## API Endpoints

Base path: `/api`

### Health

| Method | Path      | Description        | Response |
|--------|-----------|--------------------|----------|
| `GET`  | `/health` | DB connectivity check | `200 { "status": "healthy" }` or `503` |

### Podcasts

| Method   | Path                          | Description                        | Request Body         | Response                |
|----------|-------------------------------|------------------------------------|----------------------|-------------------------|
| `GET`    | `/api/podcasts`               | List podcasts (filtered, paginated)| —                    | `200 { items, pagination }` |
| `GET`    | `/api/podcasts/{id}`          | Get single podcast                 | —                    | `200 Podcast` or `404`  |
| `POST`   | `/api/podcasts`               | Create podcast                     | `CreatePodcastInput` | `201 Podcast`           |
| `PATCH`  | `/api/podcasts/{id}`          | Partial update                     | `UpdatePodcastInput` | `200 Podcast` or `404`  |
| `DELETE` | `/api/podcasts/{id}`          | Delete podcast                     | —                    | `204` or `404`          |
| `POST`   | `/api/podcasts/{id}/archive`  | Archive podcast                    | —                    | `200 Podcast` or `404`  |
| `POST`   | `/api/podcasts/{id}/unarchive`| Unarchive podcast                  | —                    | `200 Podcast` or `404`  |

#### Query parameters for `GET /api/podcasts`

| Param            | Type     | Description                              |
|------------------|----------|------------------------------------------|
| `language`       | `string` | Filter: `en`, `es`, `both`               |
| `level`          | `string` | Filter: `beginner`, `intermediate`, `advanced` |
| `country`        | `string` | Filter by country                        |
| `topic`          | `string` | Filter by topic                          |
| `includeArchived`| `bool`   | Include archived podcasts (default: false)|
| `page`           | `int`    | Page number (1-indexed)                  |
| `pageSize`       | `int`    | Items per page                           |

#### Pagination response shape

```json
{
  "items": [Podcast],
  "pagination": {
    "totalCount": 42,
    "totalPages": 5,
    "currentPage": 1
  }
}
```

### Link Reports

| Method   | Path                                | Description                     | Response                |
|----------|-------------------------------------|---------------------------------|-------------------------|
| `POST`   | `/api/podcasts/{id}/report`         | Report dead link (IP-deduplicated) | `201 LinkReport`     |
| `GET`    | `/api/podcasts/{id}/reports`        | List reports for a podcast      | `200 [LinkReport]`      |
| `GET`    | `/api/podcasts/{id}/reports/count`  | Count reports for a podcast     | `200 { "count": N }`   |
| `DELETE` | `/api/podcasts/{id}/reports`        | Clear all reports for a podcast | `204`                   |
| `GET`    | `/api/link-reports/counts`          | Report counts for all podcasts  | `200 [LinkReportCount]` |

---

## Data Types

### Podcast (JSON response)

```json
{
  "id": "uuid",
  "title": "string",
  "description": "string",
  "imageUrl": "string",
  "language": "en | es | both",
  "level": "beginner | intermediate | advanced",
  "country": "string",
  "topic": "string",
  "url": "string",
  "archived": false,
  "createdAt": "2026-01-01T00:00:00Z",
  "updatedAt": "2026-01-01T00:00:00Z"
}
```

### CreatePodcastInput

All fields required:
```json
{
  "title": "string (1-255)",
  "description": "string (1-1000)",
  "imageUrl": "valid URL",
  "language": "en | es | both",
  "level": "beginner | intermediate | advanced",
  "country": "string (1-100)",
  "topic": "string (1-100)",
  "url": "valid URL"
}
```

### UpdatePodcastInput

All fields optional (PATCH semantics):
```json
{
  "title?": "string (1-255)",
  "description?": "string (1-1000)",
  "imageUrl?": "valid URL",
  "language?": "en | es | both",
  "level?": "string",
  "country?": "string (1-100)",
  "topic?": "string (1-100)",
  "url?": "valid URL"
}
```

### LinkReport

```json
{
  "id": "uuid",
  "podcastId": "uuid",
  "reporterIp": "string | null",
  "createdAt": "2026-01-01T00:00:00Z"
}
```

### LinkReportCount

```json
{
  "podcastId": "uuid",
  "count": 5
}
```

### Error response

```json
{
  "error": "Not Found",
  "message": "podcast not found"
}
```

---

## Frontend Usage (what the frontend actually calls)

From `src/api/podcasts.ts`:

1. `GET /api/podcasts` — list all (no filters used currently)
2. `GET /api/podcasts/{id}` — single podcast detail
3. `POST /api/podcasts/{id}/report` — report dead link

The admin endpoints (create, update, delete, archive, reports listing, clear reports) are not used by the frontend but are needed for admin tooling / future use.

---

## Rust Architecture

```
backend/
├── Cargo.toml
├── Dockerfile
├── migrations/           # SQL migration files (sqlx)
│   └── 001_init.sql
└── src/
    ├── main.rs           # Entrypoint: config, DB pool, start server
    ├── config.rs         # Env-based config (DATABASE_URL, PORT, etc.)
    ├── error.rs          # AppError type, IntoResponse impl
    ├── db.rs             # Pool setup, health check
    ├── routes/
    │   ├── mod.rs        # Router assembly
    │   ├── health.rs
    │   ├── podcast.rs    # Podcast handlers
    │   └── link_report.rs# Link report handlers
    ├── models/
    │   ├── mod.rs
    │   ├── podcast.rs    # Podcast, CreatePodcastInput, UpdatePodcastInput, PodcastFilters
    │   └── link_report.rs# LinkReport, LinkReportCount
    └── repo/
        ├── mod.rs
        ├── podcast.rs    # SQL queries for podcasts
        └── link_report.rs# SQL queries for link reports
```

### Crate choices

| Concern         | Crate                  | Why                                    |
|-----------------|------------------------|----------------------------------------|
| HTTP framework  | `axum`                 | Tokio-native, ergonomic extractors     |
| Database        | `sqlx` (Postgres)      | Compile-time checked queries, async    |
| Serialization   | `serde` + `serde_json` | Standard                               |
| Validation      | `validator`            | Derive-based, similar to Go's validate |
| Config          | `dotenvy` + `std::env` | Simple, no over-engineering            |
| Logging         | `tracing` + `tracing-subscriber` | Structured, async-aware       |
| CORS            | `tower-http`           | Middleware for axum                    |
| UUID            | `uuid`                 | v4 generation                          |

### Design decisions

- No separate "service" layer — the Go version's service layer is pure passthrough (calls repo, returns result). Per YAGNI, handlers call repo directly. Add a service layer only when business logic emerges.
- No ORM — raw SQL via `sqlx::query_as!` for type safety without abstraction overhead.
- Repo functions are free functions (not methods on a struct) that take `&PgPool` — simpler than trait-based DI for this scale.
- Single `AppError` enum that implements `IntoResponse` — no scattered error handling.
- JSON field names use camelCase (matching the Go API and frontend expectations).

### Environment variables

| Variable       | Required | Default | Description          |
|----------------|----------|---------|----------------------|
| `DATABASE_URL` | Yes      | —       | Postgres connection  |
| `PORT`         | No       | `8080`  | Server listen port   |
| `RUST_LOG`     | No       | `info`  | Tracing filter       |

---

## Migration SQL

```sql
-- 001_init.sql

CREATE TABLE IF NOT EXISTS podcasts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title       VARCHAR(255) NOT NULL,
    description VARCHAR(1000) NOT NULL,
    image_url   TEXT NOT NULL,
    language    VARCHAR(10) NOT NULL CHECK (language IN ('en', 'es', 'both')),
    level       VARCHAR(20) NOT NULL CHECK (level IN ('beginner', 'intermediate', 'advanced')),
    country     VARCHAR(100) NOT NULL,
    topic       VARCHAR(100) NOT NULL,
    url         TEXT NOT NULL,
    archived    BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_podcasts_language ON podcasts (language);
CREATE INDEX IF NOT EXISTS idx_podcasts_level ON podcasts (level);
CREATE INDEX IF NOT EXISTS idx_podcasts_archived ON podcasts (archived);

CREATE TABLE IF NOT EXISTS link_reports (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    podcast_id  UUID NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE,
    reporter_ip INET,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_link_reports_dedup
    ON link_reports (podcast_id, reporter_ip)
    WHERE reporter_ip IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_link_reports_podcast_id ON link_reports (podcast_id);
```

---

## Implementation Plan

### Phase 1 — Core (frontend-facing)
1. Project scaffold: `Cargo.toml`, config, DB pool, health check
2. Podcast model + repo (`GetAll` with filters/pagination, `GetByID`)
3. `GET /api/podcasts`, `GET /api/podcasts/{id}` handlers
4. LinkReport model + repo (`Create` with IP dedup)
5. `POST /api/podcasts/{id}/report` handler
6. CORS, error handling, Dockerfile
7. Test against frontend locally

### Phase 2 — Admin endpoints
8. `POST /api/podcasts` (create)
9. `PATCH /api/podcasts/{id}` (update)
10. `DELETE /api/podcasts/{id}`
11. `POST /api/podcasts/{id}/archive` + `/unarchive`
12. `GET /api/podcasts/{id}/reports`, `/reports/count`
13. `DELETE /api/podcasts/{id}/reports`
14. `GET /api/link-reports/counts`

---

## Sources

- [Go backend source](https://github.com/Jaleel-VS/spa-eng-discord-website-backend) — accessed 2026-04-04
- [Frontend source](https://github.com/Jaleel-VS/the-spanish-english-discord-server) — accessed 2026-04-04

# Hablemos Backend

REST API for the [Spanish-English Discord](https://github.com/Jaleel-VS/the-spanish-english-discord-server) community website. Rust rewrite of the [original Go backend](https://github.com/Jaleel-VS/spa-eng-discord-website-backend).

## Stack

- [Rust](https://www.rust-lang.org/) (2024 edition)
- [Axum](https://github.com/tokio-rs/axum) — HTTP framework
- [SQLx](https://github.com/launchbadge/sqlx) — async Postgres with compile-time checked queries
- [PostgreSQL](https://www.postgresql.org/) — database
- [Docker](https://www.docker.com/) — containerized deployment (Railway)

## Setup

```bash
# Start a local Postgres (or use your own)
# Create database and user matching .env

# Run migrations and start the server
make dev
```

### Environment Variables

| Variable       | Required | Default | Description          |
|----------------|----------|---------|----------------------|
| `DATABASE_URL` | Yes      | —       | Postgres connection string |
| `PORT`         | No       | `8080`  | Server listen port   |
| `RUST_LOG`     | No       | `info`  | Tracing log filter   |

## Scripts

| Command     | Description                    |
|-------------|--------------------------------|
| `make dev`  | Run the server (`cargo run`)   |
| `make test` | Run tests (single-threaded)    |

## Database Schema

```mermaid
erDiagram
    podcasts {
        UUID id PK
        VARCHAR(255) title
        VARCHAR(1000) description
        TEXT image_url
        VARCHAR(10) language "en | es | both"
        VARCHAR(20) level "beginner | intermediate | advanced"
        VARCHAR(100) country
        VARCHAR(100) topic
        TEXT url
        BOOLEAN archived "default false"
        TIMESTAMPTZ created_at
        TIMESTAMPTZ updated_at
    }

    link_reports {
        UUID id PK
        UUID podcast_id FK
        INET reporter_ip "nullable, unique per podcast"
        TIMESTAMPTZ created_at
    }

    podcasts ||--o{ link_reports : "has many"
```

## API Endpoints

```mermaid
graph LR
    subgraph Health
        H[GET /health]
    end

    subgraph Podcasts
        LP[GET /api/podcasts]
        GP[GET /api/podcasts/:id]
        CP[POST /api/podcasts]
        UP[PATCH /api/podcasts/:id]
        DP[DELETE /api/podcasts/:id]
        AP[POST /api/podcasts/:id/archive]
        UAP[POST /api/podcasts/:id/unarchive]
    end

    subgraph Link Reports
        RP[POST /api/podcasts/:id/report]
        LR[GET /api/podcasts/:id/reports]
        CR[GET /api/podcasts/:id/reports/count]
        DR[DELETE /api/podcasts/:id/reports]
        AC[GET /api/link-reports/counts]
    end
```

### Podcasts

| Method   | Path                           | Description              | Auth  |
|----------|--------------------------------|--------------------------|-------|
| `GET`    | `/api/podcasts`                | List (filtered, paginated) | —   |
| `GET`    | `/api/podcasts/:id`            | Get by ID                | —     |
| `POST`   | `/api/podcasts`                | Create                   | Admin |
| `PATCH`  | `/api/podcasts/:id`            | Partial update           | Admin |
| `DELETE` | `/api/podcasts/:id`            | Delete                   | Admin |
| `POST`   | `/api/podcasts/:id/archive`    | Archive                  | Admin |
| `POST`   | `/api/podcasts/:id/unarchive`  | Unarchive                | Admin |

#### Query params for `GET /api/podcasts`

`language`, `level`, `country`, `topic`, `includeArchived`, `page`, `pageSize`

### Link Reports

| Method   | Path                              | Description                  | Auth  |
|----------|-----------------------------------|------------------------------|-------|
| `POST`   | `/api/podcasts/:id/report`        | Report dead link (IP-deduped) | —    |
| `GET`    | `/api/podcasts/:id/reports`       | List reports for podcast     | Admin |
| `GET`    | `/api/podcasts/:id/reports/count` | Count reports for podcast    | Admin |
| `DELETE` | `/api/podcasts/:id/reports`       | Clear reports for podcast    | Admin |
| `GET`    | `/api/link-reports/counts`        | All podcast report counts    | Admin |

## Architecture

```mermaid
graph TD
    Client[Frontend / Client] -->|HTTP| Axum[Axum Router]
    Axum --> Routes[Route Handlers]
    Routes --> Repo[Repo Layer - raw SQL]
    Repo --> PG[(PostgreSQL)]

    style Client fill:#fb923c,color:#000
    style PG fill:#336791,color:#fff
```

No service layer — handlers call repo directly (YAGNI). Repo functions are free functions taking `&PgPool`.

```
src/
├── main.rs           # Entrypoint: config, DB pool, server
├── config.rs         # Env-based config
├── error.rs          # AppError → HTTP response mapping
├── db.rs             # Pool setup, migrations, health check
├── routes/
│   ├── mod.rs        # Router assembly
│   ├── health.rs
│   ├── podcast.rs
│   └── link_report.rs
├── models/
│   ├── podcast.rs    # Podcast, CreateInput, UpdateInput, Filters
│   └── link_report.rs
└── repo/
    ├── podcast.rs    # Podcast SQL queries
    └── link_report.rs
```

## TODO

- [ ] Movies resource — new table, CRUD endpoints, filters (genre, language, country, level), pagination, search
- [ ] Admin authentication (API key or JWT)
- [ ] Rate limiting on public endpoints
- [ ] Books, courses, conversation, music resource endpoints
- [ ] Seed data script for local development
- [ ] CI pipeline (cargo test, clippy, fmt check)
- [ ] OpenAPI / Swagger spec generation

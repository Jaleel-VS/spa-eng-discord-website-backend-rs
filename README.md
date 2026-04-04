# Hablemos Backend

REST API for the [Spanish-English Discord](https://github.com/Jaleel-VS/the-spanish-english-discord-server) community website. Rust rewrite of the [original Go backend](https://github.com/Jaleel-VS/spa-eng-discord-website-backend).

## Stack

- [Rust](https://www.rust-lang.org/) (2024 edition)
- [Axum](https://github.com/tokio-rs/axum) — HTTP framework
- [SQLx](https://github.com/launchbadge/sqlx) — async Postgres with compile-time checked queries
- [PostgreSQL](https://www.postgresql.org/) — database
- [Docker](https://www.docker.com/) — containerized deployment (Railway)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition, stable)
- [PostgreSQL](https://www.postgresql.org/download/) 15+
- (Optional) [Docker](https://docs.docker.com/get-docker/) for containerized deployment

## Local Development

### 1. Set up Postgres

```bash
# macOS (Homebrew)
brew install postgresql@15
brew services start postgresql@15

# Create the database and user
psql postgres -c "CREATE USER hablemos WITH PASSWORD 'hablemos';"
psql postgres -c "CREATE DATABASE hablemos_test OWNER hablemos;"
```

Or use Docker:

```bash
docker run -d --name hablemos-pg \
  -e POSTGRES_USER=hablemos \
  -e POSTGRES_PASSWORD=hablemos \
  -e POSTGRES_DB=hablemos_test \
  -p 5432:5432 \
  postgres:15
```

### 2. Configure environment

```bash
cp .env.example .env
# Edit .env if your Postgres credentials differ
```

The default `.env` expects:
```
DATABASE_URL=postgres://hablemos:hablemos@localhost:5432/hablemos_test
```

### 3. Run

```bash
make dev
```

This runs `cargo run`, which automatically applies migrations on startup and starts the server on `http://localhost:8080`.

### 4. Verify

```bash
curl http://localhost:8080/health
# → {"status":"healthy"}
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

    movies {
        UUID id PK
        VARCHAR(255) title
        VARCHAR(2000) description
        TEXT poster_url
        VARCHAR(10) audio_language "en | es | both"
        VARCHAR(10) subtitle_language "en | es | both | none"
        VARCHAR(20) level "beginner | intermediate | advanced"
        VARCHAR(100) country
        VARCHAR(50) genre
        INT release_year
        TEXT url "streaming/info link"
        BOOLEAN archived "default false"
        TIMESTAMPTZ created_at
        TIMESTAMPTZ updated_at
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

    subgraph Movies
        LM[GET /api/movies]
        GM[GET /api/movies/:id]
        CM[POST /api/movies]
        DM[DELETE /api/movies/:id]
        DBM[DELETE /api/movies]
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

---

### Movies (planned)

| Method   | Path                | Description                        | Auth  |
|----------|---------------------|------------------------------------|-------|
| `GET`    | `/api/movies`       | List movies (filtered, paginated)  | —     |
| `GET`    | `/api/movies/:id`   | Get single movie                   | —     |
| `POST`   | `/api/movies`       | Create movie                       | Admin |
| `DELETE` | `/api/movies/:id`   | Delete movie                       | Admin |
| `DELETE` | `/api/movies`       | Bulk delete movies (body: `{ ids }`) | Admin |

#### Query params for `GET /api/movies`

| Param              | Type     | Description                                    |
|--------------------|----------|------------------------------------------------|
| `audioLanguage`    | `string` | Filter: `en`, `es`, `both`                     |
| `subtitleLanguage` | `string` | Filter: `en`, `es`, `both`, `none`             |
| `level`            | `string` | Filter: `beginner`, `intermediate`, `advanced`  |
| `genre`            | `string` | Filter by genre                                |
| `country`          | `string` | Filter by country of origin                    |
| `search`           | `string` | Full-text search on title and description      |
| `includeArchived`  | `bool`   | Include archived movies (default: false)       |
| `page`             | `int`    | Page number (1-indexed, default: 1)            |
| `pageSize`         | `int`    | Items per page (default: 20, max: 100)         |

#### Movie JSON response

```json
{
  "id": "uuid",
  "title": "Pan's Labyrinth",
  "description": "A girl escapes into a fantasy world...",
  "posterUrl": "https://...",
  "audioLanguage": "es",
  "subtitleLanguage": "en",
  "level": "advanced",
  "country": "Spain",
  "genre": "Fantasy",
  "releaseYear": 2006,
  "url": "https://...",
  "archived": false,
  "createdAt": "2026-01-01T00:00:00Z",
  "updatedAt": "2026-01-01T00:00:00Z"
}
```

#### CreateMovieInput

```json
{
  "title": "string (1-255)",
  "description": "string (1-2000)",
  "posterUrl": "valid URL",
  "audioLanguage": "en | es | both",
  "subtitleLanguage": "en | es | both | none",
  "level": "beginner | intermediate | advanced",
  "country": "string (1-100)",
  "genre": "string (1-50)",
  "releaseYear": 2006,
  "url": "valid URL"
}
```

#### Migration SQL (planned)

```sql
CREATE TABLE IF NOT EXISTS movies (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title              VARCHAR(255) NOT NULL,
    description        VARCHAR(2000) NOT NULL,
    poster_url         TEXT NOT NULL,
    audio_language     VARCHAR(10) NOT NULL CHECK (audio_language IN ('en', 'es', 'both')),
    subtitle_language  VARCHAR(10) NOT NULL CHECK (subtitle_language IN ('en', 'es', 'both', 'none')),
    level              VARCHAR(20) NOT NULL CHECK (level IN ('beginner', 'intermediate', 'advanced')),
    country            VARCHAR(100) NOT NULL,
    genre              VARCHAR(50) NOT NULL,
    release_year       INT NOT NULL,
    url                TEXT NOT NULL,
    archived           BOOLEAN NOT NULL DEFAULT false,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_movies_audio_language ON movies (audio_language);
CREATE INDEX IF NOT EXISTS idx_movies_subtitle_language ON movies (subtitle_language);
CREATE INDEX IF NOT EXISTS idx_movies_level ON movies (level);
CREATE INDEX IF NOT EXISTS idx_movies_genre ON movies (genre);
CREATE INDEX IF NOT EXISTS idx_movies_archived ON movies (archived);
```

#### Design notes

- **Two language columns** (`audio_language`, `subtitle_language`) instead of one — a movie in Spanish with English subs is a different learning experience than one dubbed into English. This lets the frontend filter by "watch in Spanish with English subtitles" which is the most common use case.
- **`subtitle_language` includes `none`** — some users want immersion without subtitles.
- **`genre` is a single string**, not a join table. Keeps it simple and consistent with how podcasts handle `topic`. If multi-genre becomes needed, migrate to an array column or join table later.
- **`search` query param** uses `ILIKE` on title/description. Good enough at this scale; add `tsvector` full-text search if the dataset grows past ~1k rows.
- **Bulk delete** (`DELETE /api/movies` with body `{ "ids": [...] }`) — useful for admin cleanup without N individual requests.
- **No update endpoint yet** — YAGNI. Movies are mostly static metadata. Add `PATCH` when there's a real need.

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

- [ ] Movies resource — implement migration, model, repo, routes (design above)
- [ ] Admin authentication (API key or JWT)
- [ ] Rate limiting on public endpoints
- [ ] Books, courses, conversation, music resource endpoints
- [ ] Seed data script for local development
- [ ] CI pipeline (cargo test, clippy, fmt check)
- [ ] OpenAPI / Swagger spec generation

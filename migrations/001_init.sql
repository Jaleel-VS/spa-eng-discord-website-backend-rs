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

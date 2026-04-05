CREATE TABLE IF NOT EXISTS movies (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title              VARCHAR(255) NOT NULL,
    description        VARCHAR(2000) NOT NULL,
    poster_url         TEXT NOT NULL,
    audio_language     VARCHAR(10) NOT NULL CHECK (audio_language IN ('en', 'es', 'both')),
    country            VARCHAR(100) NOT NULL,
    genre              VARCHAR(50) NOT NULL,
    release_year       INT NOT NULL,
    url                TEXT NOT NULL,
    archived           BOOLEAN NOT NULL DEFAULT false,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_movies_audio_language ON movies (audio_language);
CREATE INDEX IF NOT EXISTS idx_movies_genre ON movies (genre);
CREATE INDEX IF NOT EXISTS idx_movies_archived ON movies (archived);

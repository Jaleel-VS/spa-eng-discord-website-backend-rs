use sqlx::PgPool;
use uuid::Uuid;

use crate::models::movie::{CreateMovieInput, Movie, MovieFilters};

pub async fn list(pool: &PgPool, filters: &MovieFilters) -> Result<(Vec<Movie>, i64), sqlx::Error> {
    let mut where_clauses = Vec::new();
    let mut args: Vec<String> = Vec::new();

    if !filters.include_archived {
        where_clauses.push("archived = false".to_string());
    }
    if let Some(ref lang) = filters.audio_language {
        args.push(lang.clone());
        where_clauses.push(format!("audio_language = ${}", args.len()));
    }
    if let Some(ref genre) = filters.genre {
        args.push(genre.clone());
        where_clauses.push(format!("genre = ${}", args.len()));
    }
    if let Some(ref country) = filters.country {
        args.push(country.clone());
        where_clauses.push(format!("country = ${}", args.len()));
    }
    if let Some(ref search) = filters.search {
        args.push(format!("%{search}%"));
        let idx = args.len();
        where_clauses.push(format!("(title ILIKE ${idx} OR description ILIKE ${idx})"));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) as count FROM movies {where_sql}");
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for arg in &args {
        count_query = count_query.bind(arg);
    }
    let total_count = count_query.fetch_one(pool).await?;

    let mut data_sql = format!(
        "SELECT id, title, description, poster_url, audio_language, country, genre, release_year, url, archived, created_at, updated_at \
         FROM movies {where_sql} ORDER BY created_at DESC"
    );

    let page = filters.page.unwrap_or(1).max(1);
    let page_size = filters.page_size.unwrap_or(0);

    if page_size > 0 {
        let offset = (page - 1) * page_size;
        data_sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));
    }

    let mut data_query = sqlx::query_as::<_, Movie>(&data_sql);
    for arg in &args {
        data_query = data_query.bind(arg);
    }
    let movies = data_query.fetch_all(pool).await?;

    Ok((movies, total_count))
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Movie, sqlx::Error> {
    sqlx::query_as::<_, Movie>(
        "SELECT id, title, description, poster_url, audio_language, country, genre, release_year, url, archived, created_at, updated_at \
         FROM movies WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create(pool: &PgPool, input: &CreateMovieInput) -> Result<Movie, sqlx::Error> {
    sqlx::query_as::<_, Movie>(
        "INSERT INTO movies (title, description, poster_url, audio_language, country, genre, release_year, url) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id, title, description, poster_url, audio_language, country, genre, release_year, url, archived, created_at, updated_at"
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.poster_url)
    .bind(&input.audio_language)
    .bind(&input.country)
    .bind(&input.genre)
    .bind(input.release_year)
    .bind(&input.url)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM movies WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn bulk_delete(pool: &PgPool, ids: &[Uuid]) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM movies WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

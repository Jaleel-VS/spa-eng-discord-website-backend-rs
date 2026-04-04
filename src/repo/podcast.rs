use sqlx::PgPool;
use uuid::Uuid;

use crate::models::podcast::{CreatePodcastInput, Podcast, PodcastFilters, UpdatePodcastInput};

pub async fn list(pool: &PgPool, filters: &PodcastFilters) -> Result<(Vec<Podcast>, i64), sqlx::Error> {
    let mut where_clauses = Vec::new();
    let mut args: Vec<String> = Vec::new();

    if !filters.include_archived {
        where_clauses.push("archived = false".to_string());
    }
    if let Some(ref lang) = filters.language {
        args.push(lang.clone());
        where_clauses.push(format!("language = ${}", args.len()));
    }
    if let Some(ref level) = filters.level {
        args.push(level.clone());
        where_clauses.push(format!("level = ${}", args.len()));
    }
    if let Some(ref country) = filters.country {
        args.push(country.clone());
        where_clauses.push(format!("country = ${}", args.len()));
    }
    if let Some(ref topic) = filters.topic {
        args.push(topic.clone());
        where_clauses.push(format!("topic = ${}", args.len()));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Build count query
    let count_sql = format!("SELECT COUNT(*) as count FROM podcasts {where_sql}");
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for arg in &args {
        count_query = count_query.bind(arg);
    }
    let total_count = count_query.fetch_one(pool).await?;

    // Build data query
    let mut data_sql = format!(
        "SELECT id, title, description, image_url, language, level, country, topic, url, archived, created_at, updated_at \
         FROM podcasts {where_sql} ORDER BY created_at DESC"
    );

    let page = filters.page.unwrap_or(1).max(1);
    let page_size = filters.page_size.unwrap_or(0);

    if page_size > 0 {
        let offset = (page - 1) * page_size;
        data_sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));
    }

    let mut data_query = sqlx::query_as::<_, Podcast>(&data_sql);
    for arg in &args {
        data_query = data_query.bind(arg);
    }
    let podcasts = data_query.fetch_all(pool).await?;

    Ok((podcasts, total_count))
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Podcast, sqlx::Error> {
    sqlx::query_as::<_, Podcast>(
        "SELECT id, title, description, image_url, language, level, country, topic, url, archived, created_at, updated_at \
         FROM podcasts WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create(pool: &PgPool, input: &CreatePodcastInput) -> Result<Podcast, sqlx::Error> {
    sqlx::query_as::<_, Podcast>(
        "INSERT INTO podcasts (title, description, image_url, language, level, country, topic, url) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id, title, description, image_url, language, level, country, topic, url, archived, created_at, updated_at"
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.image_url)
    .bind(&input.language)
    .bind(&input.level)
    .bind(&input.country)
    .bind(&input.topic)
    .bind(&input.url)
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &PgPool, id: Uuid, input: &UpdatePodcastInput) -> Result<Podcast, sqlx::Error> {
    let mut sets = Vec::new();
    let mut args: Vec<String> = Vec::new();

    macro_rules! maybe_set {
        ($field:ident, $col:expr) => {
            if let Some(ref val) = input.$field {
                args.push(val.clone());
                sets.push(format!("{} = ${}", $col, args.len()));
            }
        };
    }

    maybe_set!(title, "title");
    maybe_set!(description, "description");
    maybe_set!(image_url, "image_url");
    maybe_set!(language, "language");
    maybe_set!(level, "level");
    maybe_set!(country, "country");
    maybe_set!(topic, "topic");
    maybe_set!(url, "url");

    if sets.is_empty() {
        return get_by_id(pool, id).await;
    }

    sets.push("updated_at = NOW()".to_string());
    args.push(id.to_string());
    let id_idx = args.len();

    let sql = format!(
        "UPDATE podcasts SET {} WHERE id = ${}::UUID \
         RETURNING id, title, description, image_url, language, level, country, topic, url, archived, created_at, updated_at",
        sets.join(", "),
        id_idx
    );

    let mut query = sqlx::query_as::<_, Podcast>(&sql);
    for arg in &args {
        query = query.bind(arg);
    }
    query.fetch_one(pool).await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM podcasts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn set_archived(pool: &PgPool, id: Uuid, archived: bool) -> Result<Podcast, sqlx::Error> {
    sqlx::query_as::<_, Podcast>(
        "UPDATE podcasts SET archived = $1, updated_at = NOW() WHERE id = $2 \
         RETURNING id, title, description, image_url, language, level, country, topic, url, archived, created_at, updated_at"
    )
    .bind(archived)
    .bind(id)
    .fetch_one(pool)
    .await
}

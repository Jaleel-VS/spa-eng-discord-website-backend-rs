use sqlx::PgPool;
use uuid::Uuid;

use crate::models::link_report::{LinkReport, LinkReportCount};

pub async fn create(pool: &PgPool, podcast_id: Uuid, reporter_ip: Option<&str>) -> Result<LinkReport, sqlx::Error> {
    sqlx::query_as::<_, LinkReport>(
        "INSERT INTO link_reports (podcast_id, reporter_ip) \
         VALUES ($1, $2::INET) \
         ON CONFLICT (podcast_id, reporter_ip) WHERE reporter_ip IS NOT NULL \
         DO UPDATE SET created_at = link_reports.created_at \
         RETURNING id, podcast_id, reporter_ip::TEXT, created_at"
    )
    .bind(podcast_id)
    .bind(reporter_ip)
    .fetch_one(pool)
    .await
}

pub async fn podcast_exists(pool: &PgPool, podcast_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM podcasts WHERE id = $1")
        .bind(podcast_id)
        .fetch_one(pool)
        .await?;
    Ok(result > 0)
}

pub async fn list_by_podcast(pool: &PgPool, podcast_id: Uuid) -> Result<Vec<LinkReport>, sqlx::Error> {
    sqlx::query_as::<_, LinkReport>(
        "SELECT id, podcast_id, reporter_ip::TEXT, created_at \
         FROM link_reports WHERE podcast_id = $1 ORDER BY created_at DESC"
    )
    .bind(podcast_id)
    .fetch_all(pool)
    .await
}

pub async fn count_by_podcast(pool: &PgPool, podcast_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM link_reports WHERE podcast_id = $1")
        .bind(podcast_id)
        .fetch_one(pool)
        .await
}

pub async fn get_all_counts(pool: &PgPool) -> Result<Vec<LinkReportCount>, sqlx::Error> {
    sqlx::query_as::<_, LinkReportCount>(
        "SELECT podcast_id, COUNT(*) as count FROM link_reports GROUP BY podcast_id ORDER BY count DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn clear_by_podcast(pool: &PgPool, podcast_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM link_reports WHERE podcast_id = $1")
        .bind(podcast_id)
        .execute(pool)
        .await?;
    Ok(())
}

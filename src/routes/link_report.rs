use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::repo;

pub async fn report(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<crate::models::link_report::LinkReport>), AppError> {
    if !repo::link_report::podcast_exists(&pool, id).await? {
        return Err(AppError::NotFound("podcast not found".into()));
    }

    let ip = extract_client_ip(&headers);
    let report = repo::link_report::create(&pool, id, ip.as_deref()).await?;
    Ok((StatusCode::CREATED, Json(report)))
}

pub async fn list_by_podcast(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::link_report::LinkReport>>, AppError> {
    let reports = repo::link_report::list_by_podcast(&pool, id).await?;
    Ok(Json(reports))
}

pub async fn count_by_podcast(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let count = repo::link_report::count_by_podcast(&pool, id).await?;
    Ok(Json(json!({"count": count})))
}

pub async fn get_all_counts(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<crate::models::link_report::LinkReportCount>>, AppError> {
    let counts = repo::link_report::get_all_counts(&pool).await?;
    Ok(Json(counts))
}

pub async fn clear_reports(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    repo::link_report::clear_by_podcast(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        return xff.split(',').next().map(|s| s.trim().to_string());
    }
    if let Some(xri) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        return Some(xri.to_string());
    }
    None
}

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::models::podcast::{
    CreatePodcastInput, PaginatedResponse, PaginationMeta, PodcastFilters, UpdatePodcastInput,
};
use crate::repo;

pub async fn list(
    State(pool): State<PgPool>,
    Query(filters): Query<PodcastFilters>,
) -> Result<Json<PaginatedResponse<crate::models::podcast::Podcast>>, AppError> {
    let (podcasts, total_count) = repo::podcast::list(&pool, &filters).await?;

    let page_size = filters.page_size.unwrap_or(0) as i64;
    let current_page = filters.page.unwrap_or(1).max(1) as i64;
    let total_pages = if page_size > 0 {
        (total_count + page_size - 1) / page_size
    } else {
        0
    };

    Ok(Json(PaginatedResponse {
        items: podcasts,
        pagination: PaginationMeta {
            total_count,
            total_pages,
            current_page,
        },
    }))
}

pub async fn get_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::podcast::Podcast>, AppError> {
    let podcast = repo::podcast::get_by_id(&pool, id).await.map_err(not_found)?;
    Ok(Json(podcast))
}

pub async fn create(
    State(pool): State<PgPool>,
    Json(input): Json<CreatePodcastInput>,
) -> Result<(StatusCode, Json<crate::models::podcast::Podcast>), AppError> {
    input.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let podcast = repo::podcast::create(&pool, &input).await?;
    Ok((StatusCode::CREATED, Json(podcast)))
}

pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdatePodcastInput>,
) -> Result<Json<crate::models::podcast::Podcast>, AppError> {
    input.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let podcast = repo::podcast::update(&pool, id, &input).await.map_err(not_found)?;
    Ok(Json(podcast))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = repo::podcast::delete(&pool, id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("podcast not found".into()))
    }
}

pub async fn archive(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::podcast::Podcast>, AppError> {
    let podcast = repo::podcast::set_archived(&pool, id, true).await.map_err(not_found)?;
    Ok(Json(podcast))
}

pub async fn unarchive(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::podcast::Podcast>, AppError> {
    let podcast = repo::podcast::set_archived(&pool, id, false).await.map_err(not_found)?;
    Ok(Json(podcast))
}

fn not_found(e: sqlx::Error) -> AppError {
    match e {
        sqlx::Error::RowNotFound => AppError::NotFound("podcast not found".into()),
        _ => AppError::Internal(e.to_string()),
    }
}

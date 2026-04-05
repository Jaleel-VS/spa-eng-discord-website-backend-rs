use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::models::movie::{BulkDeleteInput, CreateMovieInput, MovieFilters};
use crate::models::podcast::{PaginatedResponse, PaginationMeta};
use crate::repo;

pub async fn list(
    State(pool): State<PgPool>,
    Query(filters): Query<MovieFilters>,
) -> Result<Json<PaginatedResponse<crate::models::movie::Movie>>, AppError> {
    let (movies, total_count) = repo::movie::list(&pool, &filters).await?;

    let page_size = filters.page_size.unwrap_or(0) as i64;
    let current_page = filters.page.unwrap_or(1).max(1) as i64;
    let total_pages = if page_size > 0 {
        (total_count + page_size - 1) / page_size
    } else {
        0
    };

    Ok(Json(PaginatedResponse {
        items: movies,
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
) -> Result<Json<crate::models::movie::Movie>, AppError> {
    let movie = repo::movie::get_by_id(&pool, id).await.map_err(not_found)?;
    Ok(Json(movie))
}

pub async fn create(
    State(pool): State<PgPool>,
    Json(input): Json<CreateMovieInput>,
) -> Result<(StatusCode, Json<crate::models::movie::Movie>), AppError> {
    input.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let movie = repo::movie::create(&pool, &input).await?;
    Ok((StatusCode::CREATED, Json(movie)))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = repo::movie::delete(&pool, id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("movie not found".into()))
    }
}

pub async fn bulk_delete(
    State(pool): State<PgPool>,
    Json(input): Json<BulkDeleteInput>,
) -> Result<StatusCode, AppError> {
    if input.ids.is_empty() {
        return Err(AppError::BadRequest("ids must not be empty".into()));
    }
    repo::movie::bulk_delete(&pool, &input.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn not_found(e: sqlx::Error) -> AppError {
    match e {
        sqlx::Error::RowNotFound => AppError::NotFound("movie not found".into()),
        _ => AppError::Internal(e.to_string()),
    }
}

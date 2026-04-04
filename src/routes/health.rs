use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn health_check(
    State(pool): State<PgPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "Service Unavailable", "message": "database unavailable"})),
            )
        })?;

    Ok(Json(json!({"status": "healthy"})))
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub poster_url: String,
    pub audio_language: String,
    pub country: String,
    pub genre: String,
    pub release_year: i32,
    pub url: String,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovieInput {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1, max = 2000))]
    pub description: String,
    #[validate(url)]
    pub poster_url: String,
    #[validate(custom(function = "validate_audio_language"))]
    pub audio_language: String,
    #[validate(length(min = 1, max = 100))]
    pub country: String,
    #[validate(length(min = 1, max = 50))]
    pub genre: String,
    pub release_year: i32,
    #[validate(url)]
    pub url: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MovieFilters {
    pub audio_language: Option<String>,
    pub genre: Option<String>,
    pub country: Option<String>,
    pub search: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteInput {
    pub ids: Vec<Uuid>,
}

const VALID_AUDIO_LANGUAGES: &[&str] = &["en", "es", "both"];

fn validate_audio_language(val: &str) -> Result<(), validator::ValidationError> {
    if VALID_AUDIO_LANGUAGES.contains(&val) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_audio_language"))
    }
}

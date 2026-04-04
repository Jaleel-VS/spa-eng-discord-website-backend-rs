use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Podcast {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub image_url: String,
    pub language: String,
    pub level: String,
    pub country: String,
    pub topic: String,
    pub url: String,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreatePodcastInput {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1, max = 1000))]
    pub description: String,
    #[validate(url)]
    pub image_url: String,
    #[validate(custom(function = "validate_language"))]
    pub language: String,
    #[validate(custom(function = "validate_level"))]
    pub level: String,
    #[validate(length(min = 1, max = 100))]
    pub country: String,
    #[validate(length(min = 1, max = 100))]
    pub topic: String,
    #[validate(url)]
    pub url: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePodcastInput {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 1000))]
    pub description: Option<String>,
    #[validate(url)]
    pub image_url: Option<String>,
    #[validate(custom(function = "validate_language_opt"))]
    pub language: Option<String>,
    #[validate(custom(function = "validate_level_opt"))]
    pub level: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub country: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub topic: Option<String>,
    #[validate(url)]
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PodcastFilters {
    pub language: Option<String>,
    pub level: Option<String>,
    pub country: Option<String>,
    pub topic: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub total_count: i64,
    pub total_pages: i64,
    pub current_page: i64,
}

const VALID_LANGUAGES: &[&str] = &["en", "es", "both"];
const VALID_LEVELS: &[&str] = &["beginner", "intermediate", "advanced"];

fn validate_language(val: &str) -> Result<(), validator::ValidationError> {
    if VALID_LANGUAGES.contains(&val) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_language"))
    }
}

fn validate_level(val: &str) -> Result<(), validator::ValidationError> {
    if VALID_LEVELS.contains(&val) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_level"))
    }
}

fn validate_language_opt(val: &str) -> Result<(), validator::ValidationError> {
    validate_language(val)
}

fn validate_level_opt(val: &str) -> Result<(), validator::ValidationError> {
    validate_level(val)
}

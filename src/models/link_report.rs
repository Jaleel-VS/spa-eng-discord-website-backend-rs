use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LinkReport {
    pub id: Uuid,
    pub podcast_id: Uuid,
    pub reporter_ip: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LinkReportCount {
    pub podcast_id: Uuid,
    pub count: i64,
}

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::Serialize;

pub fn internal_error(e: sqlx::Error) -> (StatusCode, String) {
    tracing::error!(error = %e, "database error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal server error".to_string(),
    )
}


#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Application {
    pub applicant_amka: String,
    pub applicant_id: String,
    pub applicant_email: String,
    pub application_type: String,
    pub deferment_reason: Option<String>,
    pub service_division: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    pub review_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Shared model for all 3 roles. The version each gets is projected by the query
/// `applicant_ssn` is the table's primary key since a citizen has at most one application
/// `application_type` decides which of `deferment_reason` or `service_division` is populated
/// exactly one of the two can be non-null per row, enforced by a CHECK constraint at the database level (application_type_shape).
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Application {
    pub applicant_ssn: String,
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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{identity::Identity, models::{Application}, AppState, };

const VALID_STATUSES: [&str; 4] = ["pending", "approved", "rejected", "withdrawn"];
const VALID_ROLES: [&str; 3] = ["citizen", "officer", "admin"];

/// Admin can override status/review_note to correct potential accidents.
/// Admin CANNOT rewrite application_type, deferment_reason, or service_division, as those
/// are left up to the user. Maybe I can change that/allow citizens to edit their application.
#[derive(Debug, Deserialize)]
pub struct AdminUpdateApplication {
    pub status: Option<String>,
    pub review_note: Option<String>,
}

/// admin has unrestricted CRUD, including overriding a status an officer already set.
pub async fn list_applications(
    State(state): State<AppState>,
    _identity: Identity,
) -> Result<Json<Vec<Application>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Application>(
        r#"
        SELECT applicant_ssn, applicant_id, applicant_email, application_type,
               deferment_reason, service_division, status,
               reviewed_by, review_note, created_at, updated_at
        FROM applications
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

/// admin has unrestricted CRUD, including overriding a status an officer already set.
pub async fn update_application(
    State(state): State<AppState>,
    identity: Identity,
    Path(ssn): Path<String>,
    Json(payload): Json<AdminUpdateApplication>,
) -> Result<Json<Application>, (StatusCode, String)> {
    if let Some(ref status) = payload.status {
        if !VALID_STATUSES.contains(&status.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "invalid status value".into()));
        }
    }

    let exists: Option<String> = sqlx::query_scalar("SELECT applicant_ssn FROM applications WHERE applicant_ssn = $1")
        .bind(&ssn)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "application not found".into()));
    }

    let row = sqlx::query_as::<_, Application>(
        r#"
        UPDATE applications
        SET status = COALESCE($1, status),
            review_note = COALESCE($2, review_note),
            reviewed_by = $3
        WHERE applicant_ssn = $4
        RETURNING applicant_ssn, applicant_id, applicant_email, application_type,
                  deferment_reason, service_division, status,
                  reviewed_by, review_note, created_at, updated_at
        "#,
    )
    .bind(&payload.status)
    .bind(&payload.review_note)
    .bind(&identity.user_id)
    .bind(&ssn)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(row))
}

pub async fn delete_application(
    State(state): State<AppState>,
    _identity: Identity,
    Path(ssn): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM applications WHERE applicant_ssn = $1")
        .bind(&ssn)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "application not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

use crate::{AppState, identity::Identity, models::Application};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;

const VALID_STATUSES: [&str; 3] = ["pending", "approved", "rejected"];

/// Admin can override status/review_note to correct potential accidents.
/// Admin CANNOT rewrite application_type, deferment_reason, or service_division, as those
/// are left up to the user. Maybe I can change that/allow citizens to edit their application.
#[derive(Debug, Deserialize)]
pub struct AdminUpdateApplication {
    pub status: Option<String>,
    pub review_note: Option<String>,
}

/// Admin can inspect every application.
pub async fn list_applications(
    State(state): State<AppState>,
    _identity: Identity,
) -> Result<Json<Vec<Application>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Application>(
        r#"
        SELECT applicant_amka, applicant_id, applicant_email, application_type,
               deferment_reason, service_division, status,
               reviewed_by, review_note, created_at, updated_at
        FROM applications
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(crate::models::internal_error)?;

    Ok(Json(rows))
}

/// Admin can override status and review notes, including a decision already made by an officer.
pub async fn update_application(
    State(state): State<AppState>,
    identity: Identity,
    Path(amka): Path<String>,
    Json(payload): Json<AdminUpdateApplication>,
) -> Result<Json<Application>, (StatusCode, String)> {
    if let Some(ref status) = payload.status {
        if !VALID_STATUSES.contains(&status.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "invalid status value".into()));
        }
    }

    let exists: Option<String> =
        sqlx::query_scalar("SELECT applicant_amka FROM applications WHERE applicant_amka = $1")
            .bind(&amka)
            .fetch_optional(&state.db)
            .await
            .map_err(crate::models::internal_error)?;

    if exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "application not found".into()));
    }

    let row = sqlx::query_as::<_, Application>(
        r#"
        UPDATE applications
        SET status = COALESCE($1, status),
            review_note = COALESCE($2, review_note),
            reviewed_by = $3
        WHERE applicant_amka = $4
        RETURNING applicant_amka, applicant_id, applicant_email, application_type,
                  deferment_reason, service_division, status,
                  reviewed_by, review_note, created_at, updated_at
        "#,
    )
    .bind(&payload.status)
    .bind(&payload.review_note)
    .bind(&identity.user_id)
    .bind(&amka)
    .fetch_one(&state.db)
    .await
    .map_err(crate::models::internal_error)?;

    Ok(Json(row))
}

pub async fn delete_application(
    State(state): State<AppState>,
    _identity: Identity,
    Path(amka): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM applications WHERE applicant_amka = $1")
        .bind(&amka)
        .execute(&state.db)
        .await
        .map_err(crate::models::internal_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "application not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{AppState, identity::Identity, models::Application};

#[derive(Debug, Deserialize)]
pub struct ReviewDecision {
    /// Must be "approved" or "rejected"
    pub decision: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListFilter {
    pub status: Option<String>,
}

/// Officers can see all applications, optionally filtered by status.
pub async fn list_applications(
    State(state): State<AppState>,
    _identity: Identity,
    Query(filter): Query<ListFilter>,
) -> Result<Json<Vec<Application>>, (StatusCode, String)> {
    let rows = match filter.status {
        Some(status) => {
            sqlx::query_as::<_, Application>(
                r#"
                SELECT applicant_amka, applicant_id, applicant_email, application_type,
                       deferment_reason, service_division, status,
                       reviewed_by, review_note, created_at, updated_at
                FROM applications
                WHERE status = $1
                ORDER BY created_at ASC
                "#,
            )
            .bind(status)
            .fetch_all(&state.db)
            .await
        }
        None => {
            sqlx::query_as::<_, Application>(
                r#"
                SELECT applicant_amka, applicant_id, applicant_email, application_type,
                       deferment_reason, service_division, status,
                       reviewed_by, review_note, created_at, updated_at
                FROM applications
                ORDER BY created_at ASC
                "#,
            )
            .fetch_all(&state.db)
            .await
        }
    }
    .map_err(crate::models::internal_error)?;

    Ok(Json(rows))
}

/// Get an application by AMKA, passed in the request path.
pub async fn get_application(
    State(state): State<AppState>,
    _identity: Identity,
    Path(amka): Path<String>,
) -> Result<Json<Application>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Application>(
        r#"
        SELECT applicant_amka, applicant_id, applicant_email, application_type,
               deferment_reason, service_division, status,
               reviewed_by, review_note, created_at, updated_at
        FROM applications
        WHERE applicant_amka = $1
        "#,
    )
    .bind(amka)
    .fetch_optional(&state.db)
    .await
    .map_err(crate::models::internal_error)?;

    row.map(Json)
        .ok_or((StatusCode::NOT_FOUND, "application not found".into()))
}

/// Approve or reject a pending application. The fixed query only updates the
/// status, reviewer and review note.
pub async fn review_application(
    State(state): State<AppState>,
    identity: Identity,
    Path(amka): Path<String>,
    Json(payload): Json<ReviewDecision>,
) -> Result<Json<Application>, (StatusCode, String)> {
    if payload.decision != "approved" && payload.decision != "rejected" {
        return Err((
            StatusCode::BAD_REQUEST,
            "decision must be 'approved' or 'rejected'".into(),
        ));
    }

    let row = sqlx::query_as::<_, Application>(
        r#"
        UPDATE applications
        SET status = $1, reviewed_by = $2, review_note = $3
        WHERE applicant_amka = $4 AND status = 'pending'
        RETURNING applicant_amka, applicant_id, applicant_email, application_type,
                  deferment_reason, service_division, status,
                  reviewed_by, review_note, created_at, updated_at
        "#,
    )
    .bind(&payload.decision)
    .bind(&identity.user_id)
    .bind(&payload.note)
    .bind(&amka)
    .fetch_optional(&state.db)
    .await
    .map_err(crate::models::internal_error)?;

    if let Some(row) = row {
        return Ok(Json(row));
    }

    let existing_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM applications WHERE applicant_amka = $1")
            .bind(&amka)
            .fetch_optional(&state.db)
            .await
            .map_err(crate::models::internal_error)?;

    match existing_status {
        Some(status) => Err((
            StatusCode::CONFLICT,
            format!("application already {status}; cannot re-review"),
        )),
        None => Err((StatusCode::NOT_FOUND, "application not found".into())),
    }
}

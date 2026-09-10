use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::{AppState, identity::Identity, models::Application};

/// Defines how the incoming request JSON Payload will be parsed and validated.
#[derive(Debug, Deserialize)]
#[serde(tag = "application_type", rename_all = "snake_case")] // TODO: May not find the variant names without 'snake_case'
pub enum CreateApplication {
    Deferment { reason: String }, // If deferment = reason string required
    Service { division: String }, // If service = division string required
}

const VALID_DIVISIONS: [&str; 3] = ["land", "navy", "airforce"];
const SQL_UNIQUE_VIOLATION_CODE: &str = "23505";

/// Citizens only ever see their own application. AMKA doesn't come from a path,
/// instead directly from the identity extractor.
pub async fn get_application(
    State(state): State<AppState>,
    identity: Identity,
) -> Result<Json<Application>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Application>(
        r#"
        SELECT applicant_amka, applicant_id, applicant_email, application_type,
               deferment_reason, service_division, status,
               NULL::text AS reviewed_by, review_note, created_at, updated_at
        FROM applications
        WHERE applicant_amka = $1
        "#,
    )
    .bind(&identity.amka)
    .fetch_optional(&state.db)
    .await
    .map_err(crate::models::internal_error)?;

    // If a database entry with that AMKA is not found.
    row.map(Json)
        .ok_or((StatusCode::NOT_FOUND, "no application submitted".into()))
}

/// Creates the citizen's application.
/// A citizen cannot create another application while one for their AMKA exists. This is checked
/// here for a clean error response and enforced by the table's primary key on applicant_amka.
/// Deleting the existing row allows a later submission.
///
/// Fails with 409 if one already exists for this AMKA.
pub async fn create_application(
    State(state): State<AppState>,
    identity: Identity,
    Json(payload): Json<CreateApplication>,
) -> Result<(StatusCode, Json<Application>), (StatusCode, String)> {
    let existing: Option<String> =
        sqlx::query_scalar("SELECT applicant_amka FROM applications WHERE applicant_amka = $1")
            .bind(&identity.amka)
            .fetch_optional(&state.db) // optional allows fetch to return nothing.
            .await
            .map_err(crate::models::internal_error)?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            "an application already exists for your AMKA; it must be deleted before you can submit another".into(),
        ));
    }

    // Prepares variables for db insertion after validating their contents.
    let (application_type, deferment_reason, service_division): (
        &str,
        Option<String>,
        Option<String>,
    ) = match payload {
        CreateApplication::Deferment { reason } => {
            if reason.trim().is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "a deferment reason is required".into(),
                ));
            }
            ("deferment", Some(reason), None)
        }
        CreateApplication::Service { division } => {
            if !VALID_DIVISIONS.contains(&division.as_str()) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("division must be one of: {}", VALID_DIVISIONS.join(", ")),
                ));
            }
            ("service", None, Some(division))
        }
    };

    let row = sqlx::query_as::<_, Application>(
        r#"
        INSERT INTO applications (applicant_amka, applicant_id, applicant_email, application_type,
                                   deferment_reason, service_division)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING applicant_amka, applicant_id, applicant_email, application_type,
                  deferment_reason, service_division, status,
                  NULL::text AS reviewed_by, review_note, created_at, updated_at
        "#,
    )
    .bind(&identity.amka)
    .bind(&identity.user_id)
    .bind(&identity.email)
    .bind(application_type)
    .bind(&deferment_reason)
    .bind(&service_division)
    .fetch_one(&state.db)
    .await
    // There's a chance we get unique_violation here, that wasn't caught above due to a race condition.
    // We use this block bellow to map said db error to an HTTP-friendly error.
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e && db_err.code().as_deref() == Some(SQL_UNIQUE_VIOLATION_CODE) {
            return (
                StatusCode::CONFLICT,
                "an application already exists for your AMKA; it must be deleted before you can submit another".into(),
            );
        }
        crate::models::internal_error(e)
    })?;

    Ok((StatusCode::CREATED, Json(row)))
}

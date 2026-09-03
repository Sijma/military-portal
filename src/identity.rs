use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use crate::AppState;

/// Identity derived from headers the gateway injects after validating the session against Keycloak.
/// This binary does NOT verify a JWT itself. It trusts these headers, which only holds because
/// this process is meant to be reachable only from the gateway.
#[derive(Debug, Clone)]
pub struct Identity {
    pub user_id: String, // Keycloak sub
    pub email: String,
    pub role: String,

    /// Keycloak's custom "ssn" claim. Only citizen handlers use it (applications table's
    /// primary key), but every role must send it.
    pub ssn: String,
}

impl FromRequestParts<AppState> for Identity {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let header = |name: &str| -> Option<String> {
            parts
                .headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        };

        let user_id = header("x-user-id")
            .ok_or((StatusCode::UNAUTHORIZED, "missing X-User-Id header from gateway"))?;
        let email = header("x-user-email")
            .ok_or((StatusCode::UNAUTHORIZED, "missing X-User-Email header from gateway"))?;
        let role = header("x-user-roles")
            .ok_or((StatusCode::UNAUTHORIZED, "missing X-User-Roles header from gateway"))?;
        let ssn = header("x-user-ssn")
            .filter(|s| !s.is_empty())
            .ok_or((StatusCode::UNAUTHORIZED, "missing X-User-Ssn header from gateway."))?;

        if role != state.expected_role {
            // Gateway shouldn't route here otherwise — sanity check.
            return Err((StatusCode::FORBIDDEN, "role not permitted on this service instance"));
        }

        Ok(Identity { user_id, email, role, ssn })
    }
}

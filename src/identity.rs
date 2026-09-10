use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

use crate::AppState;

/// Identity derived from headers the gateway injects after validating the session against Keycloak.
/// This binary does NOT verify a JWT itself. It trusts these headers, which only holds because
/// this process is meant to be reachable only from the gateway.
#[derive(Debug, Clone)]
pub struct Identity {
    pub user_id: String, // Keycloak sub
    pub email: String,

    /// Keycloak's custom "amka" claim. Only citizens have one, and it's required below only for
    /// the citizen role, since it's that service's primary key.
    /// officer and admin just carry an empty string.
    pub amka: String,
}

impl FromRequestParts<AppState> for Identity {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = |name: &str| -> Option<String> {
            parts
                .headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        };

        let user_id = header("x-user-id").ok_or((
            StatusCode::UNAUTHORIZED,
            "missing X-User-Id header from gateway",
        ))?;
        let email = header("x-user-email").ok_or((
            StatusCode::UNAUTHORIZED,
            "missing X-User-Email header from gateway",
        ))?;
        let role = header("x-user-role") // x-user-role is the one role app-auth.lua matched for this request.
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "missing X-User-Role header from gateway",
            ))?;
        let amka = header("x-user-amka").unwrap_or_default();

        if role != state.expected_role {
            // Gateway shouldn't route here at all.
            return Err((
                StatusCode::FORBIDDEN,
                "role not permitted on this service instance",
            ));
        }

        if role == "citizen" && amka.is_empty() {
            return Err((
                StatusCode::UNAUTHORIZED,
                "missing X-User-Amka header from gateway",
            ));
        }

        Ok(Identity {
            user_id,
            email,
            amka,
        })
    }
}

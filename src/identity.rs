use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use crate::AppState;

/// Identity derived from headers injected by APISIX after it has already
/// validated the JWT against Keycloak (openid-connect plugin) and decided,
/// from the realm role claim, which deployed instance of this backend
/// should receive the request.
///
/// This binary does NOT verify a JWT signature, it trusts these
/// headers completely. That trust is only valid because this process
/// SHOULD only be reachable from the APISIX gateway.
///
/// The role this running instance expects is read once at startup into
/// `AppState::expected_role` (as defined from the SERVICE_ROLE env var)
/// rather than being hardcoded per file
#[derive(Debug, Clone)]
pub struct Identity {
    pub user_id: String, // Keycloak ID
    pub email: String,
    pub role: String,

    /// Keycloak's custom "ssn" claim, forwarded as X-User-Ssn
    /// see apisix/apisix.yaml.j2 and the protocol mapper in keycloak/civic-portal-realm.json
    /// Present for every role, but only the citizen handlers actually use it: it's the applications
    /// table's primary key, since a citizen has at most one application, keyed by their SSN.
    pub ssn: String,
}

/// Here we define a custom extractor so that we can abstract the Identity retrieval, allowing
/// it to be used easily by the handlers later.
///
/// More specifically, bellow we define how should Axum turn HTTP request metadata into our
/// Identity struct from above.
impl FromRequestParts<AppState> for Identity {
    // Defines what happens if extraction fails. Axum turns it into an actual HTTP Response.
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // This is a 'Closure' aka Lambda. Used to extract each metadata header bellow.
        let header = |name: &str| -> Option<String> {
            parts
                .headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        };

        let user_id = header("x-user-id")
            // Trailing question mark is an early return, in case one of the checks fails.
            .ok_or((StatusCode::UNAUTHORIZED, "missing X-User-Id header from gateway"))?;
        let email = header("x-user-email")
            .ok_or((StatusCode::UNAUTHORIZED, "missing X-User-Email header from gateway"))?;
        let role = header("x-user-roles")
            .ok_or((StatusCode::UNAUTHORIZED, "missing X-User-Roles header from gateway"))?;
        let ssn = header("x-user-ssn")
            .filter(|s| !s.is_empty())
            .ok_or((StatusCode::UNAUTHORIZED, "missing X-User-Ssn header from gateway. is the ssn protocol mapper configured in Keycloak?"))?;

        if role != state.expected_role {
            // The gateway should never route traffic here if that's the case. Sanity Check.
            return Err((StatusCode::FORBIDDEN, "role not permitted on this service instance"));
        }

        Ok(Identity { user_id, email, role, ssn })
    }
}

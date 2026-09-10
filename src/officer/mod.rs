use axum::{
    Router,
    routing::{get, put},
};

use crate::AppState;

mod handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/applications", get(handlers::list_applications))
        .route("/applications/{amka}", get(handlers::get_application))
        .route(
            "/applications/{amka}/review",
            put(handlers::review_application),
        )
}

use axum::{
    routing::{get, put},
    Router,
};

use crate::AppState;

mod handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/applications", get(handlers::list_applications))
        .route("/applications/:ssn", get(handlers::get_application))
        .route("/applications/:ssn/review", put(handlers::review_application))
}

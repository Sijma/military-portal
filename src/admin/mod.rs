use axum::{
    routing::{get, put},
    Router,
};

use crate::AppState;

mod handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/applications", get(handlers::list_applications))
        .route(
            "/applications/:ssn",
            put(handlers::update_application).delete(handlers::delete_application)
        )
}

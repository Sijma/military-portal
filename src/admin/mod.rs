use axum::{
    Router,
    routing::{get, put},
};

use crate::AppState;

mod handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/applications", get(handlers::list_applications))
        .route(
            "/applications/{amka}",
            put(handlers::update_application).delete(handlers::delete_application),
        )
}

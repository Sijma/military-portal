use crate::AppState;
use axum::{
    Router,
    routing::{get, post},
};

mod handlers;

pub fn router() -> Router<AppState> {
    // Router of type AppState
    Router::new()
        .route("/application", get(handlers::get_application))
        .route("/application", post(handlers::create_application))
}

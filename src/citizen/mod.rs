use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;

mod handlers;

pub fn router() -> Router<AppState> { // Router of type AppState
    Router::new()
        .route("/application", get(handlers::get_application))
        .route("/application", post(handlers::create_application))
}

mod citizen;
mod identity;
mod models;
mod officer;
mod admin;

use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower_http::trace::TraceLayer;

/// Shared app state. Deployment-agnostic. Everything comes from env vars set by whichever deploy
/// target is running this binary.
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub expected_role: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let service_role = env::var("SERVICE_ROLE")
        .expect("SERVICE_ROLE must be set to one of: citizen, officer, admin");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bind_addr = env::var("BIND_ADDR").expect("BIND_ADDR must be set, e.g. 0.0.0.0:8080");

    if !["citizen", "officer", "admin"].contains(&service_role.as_str()) {
        panic!("SERVICE_ROLE must be one of: citizen, officer, admin (got '{service_role}')");
    }

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");

    let state = AppState {
        db,
        expected_role: service_role.clone(),
    };

    let role_routes = match service_role.as_str() {
        "citizen" => citizen::router(),
        "officer" => officer::router(),
        "admin" => admin::router(),
        _ => unreachable!("validated above"),
    };

    let health_role = service_role.clone();
    let app = Router::new()
        .route("/health", get(move || async move { format!("{health_role}-service ok") }))
        .merge(role_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {bind_addr}: {e}"));
    tracing::info!("civic-backend (role={service_role}) listening on {bind_addr}");
    axum::serve(listener, app).await.unwrap();
}

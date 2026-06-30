mod citizen;
mod identity;
mod models;

use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower_http::trace::TraceLayer;

/// Shared application state.
/// This his binary is deployment/environment agnostic. Nothing in here is deployment or
/// environment-specific. It's all just values read from the environment at startup.
/// The deployment tool used is responsible for setting these env vars correctly.
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub gateway_secret: String,
    
    pub expected_role: String, // Which role this running instance serves, for easy access (citizen, officer, admin).
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt() // Simple logger/telemetry for debugging.
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()) // allows verbosity choice through RUST_LOG env.
        .init();

    // Validation block here while retrieving env vars.
    let service_role = env::var("SERVICE_ROLE")
        .expect("SERVICE_ROLE must be set to one of: citizen, officer, admin");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let gateway_secret = env::var("GATEWAY_SHARED_SECRET").expect("GATEWAY_SHARED_SECRET must be set");
    let bind_addr = env::var("BIND_ADDR").expect("BIND_ADDR must be set, e.g. 0.0.0.0:8080");

    if !["citizen", "officer", "admin"].contains(&service_role.as_str()) {
        panic!("SERVICE_ROLE must be one of: citizen, officer, admin (got '{service_role}')");
    }

    // Creating DB Connection using validated env variables
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");

    // Creating AppState and passing the db connection to it. This way we can re-use the existing
    // connection instead of making a new one for each CRUD operation.
    let state = AppState {
        db,
        gateway_secret,
        expected_role: service_role.clone(),
    };

    // TODO: Uncomment this when the rest of the handlers are implemented
    // The role passed in env selects which route table this instance exposes when ran.
    // let role_routes = match service_role.as_str() {
    //     "citizen" => citizen::router(),
    //     "officer" => officer::router(),
    //     "admin" => admin::router(),
    //     _ => unreachable!("validated above"), // Required by rust comp as all cases need handling.
    // };

    // TODO: comment this when the rest of the handlers are implemented above
    let role_routes = citizen::router();

    let health_role = service_role.clone();
    let app = Router::new() // TODO: Break Down layers.
        .route("/health", get(move || async move { format!("{health_role}-service ok") }))
        .merge(role_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Creating a listener based on passed bind_addr and serving the app on it.
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {bind_addr}: {e}"));
    tracing::info!("civic-backend (role={service_role}) listening on {bind_addr}");
    axum::serve(listener, app).await.unwrap();
}

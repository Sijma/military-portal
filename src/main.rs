mod admin;
mod citizen;
mod email;
mod identity;
mod models;
mod officer;

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use std::{env, process};
use tower_http::trace::TraceLayer;

const USAGE: &str =
    "Usage: military-portal --service-role <citizen|officer|admin> --bind-addr <address>";

struct CliArgs {
    service_role: String,
    bind_addr: String,
}

impl CliArgs {
    fn parse() -> Self {
        let mut service_role = None;
        let mut bind_addr = None;
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            if matches!(arg.as_str(), "-h" | "--help") {
                println!("{USAGE}");
                process::exit(0);
            }

            let value = args
                .next()
                .unwrap_or_else(|| cli_error(&format!("missing value for {arg}")));
            match arg.as_str() {
                "--service-role" => service_role = Some(value),
                "--bind-addr" => bind_addr = Some(value),
                _ => cli_error(&format!("unknown argument: {arg}")),
            }
        }

        let service_role = service_role.unwrap_or_else(|| cli_error("--service-role is required"));
        if !["citizen", "officer", "admin"].contains(&service_role.as_str()) {
            cli_error(&format!(
                "--service-role must be one of: citizen, officer, admin (got '{service_role}')"
            ));
        }

        Self {
            service_role,
            bind_addr: bind_addr.unwrap_or_else(|| cli_error("--bind-addr is required")),
        }
    }
}

fn cli_error(message: &str) -> ! {
    eprintln!("error: {message}\n\n{USAGE}");
    process::exit(2);
}

/// Shared app state. Deployment-agnostic configuration comes from the CLI and environment.
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub email: email::EmailService,
    pub expected_role: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let CliArgs {
        service_role,
        bind_addr,
    } = CliArgs::parse();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");
    let email = email::EmailService::from_env().expect("failed to configure email");

    let state = AppState {
        db,
        email,
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
        .route(
            "/health",
            get(move || async move { format!("{health_role}-service ok") }),
        )
        .merge(role_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {bind_addr}: {e}"));
    tracing::info!("military-backend (role={service_role}) listening on {bind_addr}");
    axum::serve(listener, app).await.unwrap();
}

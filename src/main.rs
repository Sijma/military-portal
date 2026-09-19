mod admin;
mod citizen;
mod email;
mod identity;
mod models;
mod officer;

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::IpAddr, net::SocketAddr, process};
use tower_http::trace::TraceLayer;

const SERVICE_ROLES: [&str; 3] = ["citizen", "officer", "admin"];
const USAGE: &str = "Usage: military-portal \
    --service-instance <citizen|officer|admin>:<port> \
    --bind-host <IP address>";

struct CliArgs {
    service_role: String,
    bind_addr: SocketAddr,
}

impl CliArgs {
    fn parse() -> Self {
        let mut service_instance = None;
        let mut bind_host = None;
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
                "--service-instance" => service_instance = Some(value),
                "--bind-host" => bind_host = Some(value),
                _ => cli_error(&format!("unknown argument: {arg}")),
            }
        }

        let service_instance =
            service_instance.unwrap_or_else(|| cli_error("--service-instance is required"));
        let (service_role, port) =
            parse_service_instance(&service_instance).unwrap_or_else(|message| cli_error(&message));

        let bind_host = bind_host.unwrap_or_else(|| cli_error("--bind-host is required"));
        let bind_host = bind_host
            .parse::<IpAddr>()
            .unwrap_or_else(|_| cli_error("--bind-host must be an IPv4 or IPv6 address"));

        Self {
            service_role,
            bind_addr: SocketAddr::new(bind_host, port),
        }
    }
}

fn parse_service_instance(value: &str) -> Result<(String, u16), String> {
    let (role, port) = value
        .rsplit_once(':')
        .ok_or_else(|| "--service-instance must have the form <role>:<port>".to_string())?;

    if !SERVICE_ROLES.contains(&role) {
        return Err(format!(
            "service role must be one of: {} (got '{role}')",
            SERVICE_ROLES.join(", ")
        ));
    }

    let port = port
        .parse::<u16>()
        .map_err(|_| format!("service port must be an integer from 1 to 65535 (got '{port}')"))?;
    if port == 0 {
        return Err("service port must be an integer from 1 to 65535 (got '0')".to_string());
    }

    Ok((role.to_string(), port))
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

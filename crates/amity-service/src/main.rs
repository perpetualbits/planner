// main.rs — amity-service binary entrypoint.
//
// Responsibilities:
//   1. Initialise structured logging (tracing-subscriber).
//   2. Load configuration from the platform config file (or defaults).
//   3. Open (and migrate) the database.
//   4. Build the axum application.
//   5. Bind to the configured address and start serving.
//
// Everything except the bind/serve step lives in the library crate so that
// integration tests can reuse the same application setup without a network.

use amity_service::{build_app, config::load_config};
use amity_storage::connection::open_database;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise structured logging. The `RUST_LOG` environment variable
    // controls the filter (e.g. `RUST_LOG=amity_service=debug`).
    // Default to `info` so the service is observable without extra config.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = load_config()?;

    tracing::info!(
        bind = %config.server.bind_address,
        port = config.server.port,
        db   = %config.database.url,
        "amity-service starting"
    );

    // Open the database and apply any pending migrations.
    // Migrations are embedded at compile time; a fresh database is fully set up
    // on first run without any manual migration step.
    let db = open_database(&config.database.url).await?;

    let app = build_app(db);

    // Construct the bind address from config. Loopback-only by default.
    let bind_addr = format!("{}:{}", config.server.bind_address, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    tracing::info!(address = %bind_addr, "listening");

    axum::serve(listener, app).await?;

    Ok(())
}

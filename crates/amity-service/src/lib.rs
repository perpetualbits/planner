// lib.rs — amity-service library root.
//
// Exposes the axum application as a library so integration tests can build it
// without launching a real OS process. The `main.rs` binary calls `build_app`
// with a real config; tests call it with an in-memory database.
//
// Modules:
//   config  — configuration loading from TOML
//   api     — HTTP handler modules (one per entity)

pub mod api;
pub mod config;

use axum::Router;
use axum::routing::{get, post};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

/// Shared state injected into every axum handler via `State<AppState>`.
///
/// Using a single state struct means handlers have a typed handle on everything
/// they need without threading individual arguments through every layer.
#[derive(Debug, Clone)]
pub struct AppState {
    /// The database connection pool. Cloning the pool is cheap — it wraps an
    /// `Arc` internally and shares the underlying connections.
    pub db: SqlitePool,
}

/// Build the axum `Router` with all routes wired up and state injected.
///
/// Extracting this into a function (rather than building inline in `main`)
/// allows integration tests to call it with a test database without spawning a
/// real network socket.
///
/// The `TraceLayer` wraps every request in a tracing span, which gives
/// structured logs (method, path, status, latency) with zero handler boilerplate.
pub fn build_app(db: SqlitePool) -> Router {
    let state = AppState { db };

    Router::new()
        // Inbox endpoints — see api/inbox.rs for handler documentation.
        .route("/api/v1/inbox", post(api::inbox::capture_inbox_item))
        .route("/api/v1/inbox/recent", get(api::inbox::list_recent))
        // Attach tracing middleware so every request is logged automatically.
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

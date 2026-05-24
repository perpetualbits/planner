// connection.rs — database connection pool and migration setup.
//
// This module is the single entry point for database initialisation. It:
//   1. Creates a `SqlitePool` from a file path or `:memory:`.
//   2. Applies pending migrations from the embedded `migrations/` directory.
//
// The pool is created with `SqlitePoolOptions` so that connection limits and
// timeouts can be tuned without changing call sites.
//
// Callers (amity-service, integration tests) call `open_database` once at
// startup and pass the returned pool to every repository function. There is
// no singleton or global pool — each call site owns its pool explicitly,
// consistent with the no-globals rule in the coding guidelines.

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::StorageError;

/// Open (or create) a `SQLite` database at `database_url` and apply all pending
/// migrations.
///
/// `database_url` may be:
///   - A file path: `"sqlite:///home/amity/amity.db"` or `"sqlite://./amity.db"`
///   - In-memory for tests: `"sqlite::memory:"`
///
///
/// Migrations are embedded at compile time from `crates/amity-storage/migrations/`
/// via `sqlx::migrate!`. A fresh database has all migrations applied in order;
/// an existing database has only new (unapplied) migrations applied.
///
/// # Errors
///
/// Returns `StorageError::Database` if the connection fails or a migration
/// cannot be applied. Migration failures are not recoverable — the service
/// should not start if the schema cannot be brought up to date.
pub async fn open_database(database_url: &str) -> Result<SqlitePool, StorageError> {
    // Parse the URL into connection options so we can enable WAL mode and
    // foreign-key enforcement at the session level. These pragmas are sticky
    // for file-backed databases but must be set again for each connection.
    // `FromStr` for `SqliteConnectOptions` already returns `sqlx::Error`,
    // which converts to `StorageError::Database` via the `#[from]` impl.
    let connect_options = database_url
        .parse::<SqliteConnectOptions>()?
        // Create the database file if it does not exist. Without this flag,
        // the first run would fail with "unable to open database file".
        .create_if_missing(true)
        // WAL (Write-Ahead Logging) allows concurrent reads during a write,
        // which matters for the hub where the Tauri frontend and the service
        // may read while a background sync is writing.
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        // Enforce FK constraints per connection. SQLite disables them by
        // default for backwards compatibility; we need them active.
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        // Single writer, multiple readers via WAL. A pool of 1 prevents
        // write contention while still allowing connection reuse.
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Apply all pending migrations. sqlx records applied migrations in the
    // `_sqlx_migrations` table and skips already-applied ones.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

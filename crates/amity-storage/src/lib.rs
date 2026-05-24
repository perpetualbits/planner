// lib.rs — amity-storage public API.
//
// This crate owns all persistence concerns for Amity:
//   • Database connection pool setup and migration application.
//   • Repository functions per entity (currently: inbox).
//
// The storage layer is deliberately dumb — no business logic lives here.
// Business logic belongs in amity-core; the service layer (amity-service)
// orchestrates calls between the two.
//
// All public functions accept a `&SqlitePool` rather than a wrapper type so
// that callers control connection-pool lifecycle and tests can spin up a
// fresh pool against a temp file without any global state.
//
// Modules:
//   connection  — pool construction and migration application
//   inbox       — repository functions for InboxItem

/// Database connection pool construction and migration application.
pub mod connection;

/// Repository functions for [`amity_core::inbox::InboxItem`].
pub mod inbox;

// Re-export the error type at the crate root so callers only need one import.
pub use error::StorageError;

/// Error types for the storage layer.
///
/// Every public function in this crate returns `Result<_, StorageError>`.
/// The storage error wraps sqlx errors but never exposes them raw — that
/// would tie callers to sqlx's internal error structure unnecessarily.
pub mod error {
    /// Errors produced by amity-storage repository functions.
    #[derive(Debug, thiserror::Error)]
    pub enum StorageError {
        /// A sqlx database operation failed.
        ///
        /// The inner error is the raw sqlx error. Callers that want to
        /// distinguish "not found" from "constraint violation" should inspect
        /// `sqlx::Error` variants; most callers treat this as opaque.
        #[error("database error: {0}")]
        Database(#[from] sqlx::Error),

        /// A sqlx migration failed.
        ///
        /// `MigrateError` is a distinct type from `sqlx::Error` even though
        /// it represents a database failure. We wrap it separately so the error
        /// message is clear about the migration context.
        #[error("migration error: {0}")]
        Migration(#[from] sqlx::migrate::MigrateError),

        /// A value stored in the database could not be parsed back into a
        /// domain type. This indicates either a bug in the write path or
        /// a migration that changed a column's set of valid values.
        #[error("parse error reading from database: {0}")]
        Parse(String),
    }
}

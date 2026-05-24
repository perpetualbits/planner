// inbox.rs — repository functions for InboxItem.
//
// This module is the storage layer's interface for the Inbox entity.
// It exposes three functions matching the service's needs:
//   • insert_inbox_item   — write a new item to the database
//   • fetch_inbox_item    — read one item by ID
//   • list_recent_inbox_items — read the N most recent items
//
// Design constraints (from the task spec and rust guidelines):
//   • No business logic here. The repository writes and reads domain types;
//     it does not validate, transform, or make decisions.
//   • We use runtime `sqlx::query` / `sqlx::query_as` rather than the
//     compile-time `query!` macro because the offline query cache has not been
//     generated yet. A follow-up task will run `cargo sqlx prepare` and switch
//     to the checked macros for schema-mismatch safety at compile time.
//   • UUIDs are stored as TEXT; the conversions happen explicitly in this module.
//   • OffsetDateTime is stored as ISO-8601 TEXT via time's Rfc3339 formatter.

use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use amity_core::ids::{InboxItemId, MemberId};
use amity_core::inbox::{InboxItem, InboxSource, TriageState, TypedEntityRef};

use crate::StorageError;

// ─── Row type for sqlx query_as ──────────────────────────────────────────────

/// Raw database row from the `inbox_items` table.
///
/// sqlx maps column names to field names by convention. All fields are `String`
/// or `Option<String>` because we store everything as TEXT; the conversion to
/// domain types happens in `row_to_inbox_item`.
#[derive(sqlx::FromRow)]
struct InboxItemRow {
    id: String,
    raw_text: String,
    captured_by: String,
    captured_at: String,
    source: String,
    triage_state: String,
    triaged_to: Option<String>,
}

// ─── Public repository functions ─────────────────────────────────────────────

/// Insert a new inbox item into the database.
///
/// The item's `id` and all fields are taken from the `InboxItem` struct; this
/// function does not generate IDs or timestamps — that is the service's job.
///
/// # Errors
///
/// Returns `StorageError::Database` on any sqlx failure (constraint violation,
/// I/O error, pool exhaustion).
pub async fn insert_inbox_item(pool: &SqlitePool, item: &InboxItem) -> Result<(), StorageError> {
    // Serialise the domain types to their TEXT representations for storage.
    let id = item.id.to_string();
    let captured_by = item.captured_by.to_string();
    // RFC 3339 is a profile of ISO-8601 that SQLite's text affinity sorts
    // correctly because it uses a fixed-width format with Z or ±HH:MM offset.
    let captured_at = item
        .captured_at
        .format(&Rfc3339)
        .map_err(|e| StorageError::Parse(e.to_string()))?;
    let source = item.source.to_string();
    let triage_state = item.triage_state.to_string();
    // triaged_to is NULL when absent — straightforward Option→NULL mapping.
    let triaged_to = item.triaged_to.as_ref().map(|r| r.0.clone());

    sqlx::query(
        "
        INSERT INTO inbox_items
            (id, raw_text, captured_by, captured_at, source, triage_state, triaged_to)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
    )
    .bind(id)
    .bind(&item.raw_text)
    .bind(captured_by)
    .bind(captured_at)
    .bind(source)
    .bind(triage_state)
    .bind(triaged_to)
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch a single inbox item by its ID.
///
/// Returns `None` if no item with that ID exists in the database.
/// "Not found" is a valid result, not an error — the caller decides what to
/// do with a missing item.
///
/// # Errors
///
/// Returns `StorageError::Database` on sqlx failure.
/// Returns `StorageError::Parse` if a stored value cannot be decoded into its
/// domain type (e.g. an unknown `source` string added by a newer binary).
pub async fn fetch_inbox_item(
    pool: &SqlitePool,
    id: InboxItemId,
) -> Result<Option<InboxItem>, StorageError> {
    let id_str = id.to_string();

    let row: Option<InboxItemRow> = sqlx::query_as(
        "
        SELECT id, raw_text, captured_by, captured_at, source, triage_state, triaged_to
        FROM   inbox_items
        WHERE  id = ?1
        ",
    )
    .bind(id_str)
    .fetch_optional(pool)
    .await?;

    // If the row does not exist, return None immediately. This avoids
    // parsing work for the common "item not found" path.
    match row {
        Some(r) => Ok(Some(row_to_inbox_item(r)?)),
        None => Ok(None),
    }
}

/// List the most recent inbox items, newest first.
///
/// `limit` caps the number of rows returned. Pass `20` for the default view.
/// The API layer enforces a maximum of 100 (see `amity-service::api::inbox`).
///
/// # Errors
///
/// Returns `StorageError::Database` on sqlx failure.
/// Returns `StorageError::Parse` if any stored row contains an unrecognised
/// enum value.
pub async fn list_recent_inbox_items(
    pool: &SqlitePool,
    limit: u32,
) -> Result<Vec<InboxItem>, StorageError> {
    // Cast limit to i64 for sqlx — SQLite LIMIT is a signed 64-bit integer.
    let limit_i64 = i64::from(limit);

    let rows: Vec<InboxItemRow> = sqlx::query_as(
        "
        SELECT id, raw_text, captured_by, captured_at, source, triage_state, triaged_to
        FROM   inbox_items
        ORDER  BY captured_at DESC
        LIMIT  ?1
        ",
    )
    .bind(limit_i64)
    .fetch_all(pool)
    .await?;

    // Convert each row. Collect errors (parse failures) rather than silently
    // skipping bad rows — a parse failure indicates a data integrity issue.
    rows.into_iter().map(row_to_inbox_item).collect()
}

// ─── Private helpers ─────────────────────────────────────────────────────────

/// Convert a raw database row into an `InboxItem`.
///
/// Extracted as a helper so the two query sites (`fetch_inbox_item`,
/// `list_recent_inbox_items`) share the same parsing logic and any fix
/// lands in one place.
fn row_to_inbox_item(row: InboxItemRow) -> Result<InboxItem, StorageError> {
    // Parse each TEXT column back into its domain type. Parse errors use the
    // column name in the message so debugging is easier.
    let id = row
        .id
        .parse::<InboxItemId>()
        .map_err(|e| StorageError::Parse(format!("id: {e}")))?;

    let captured_by = row
        .captured_by
        .parse::<MemberId>()
        .map_err(|e| StorageError::Parse(format!("captured_by: {e}")))?;

    // OffsetDateTime::parse with Rfc3339 requires the string to be a valid
    // RFC 3339 timestamp. All values written by this module use Rfc3339 on
    // the write path, so a parse failure here is a data integrity error.
    let captured_at = OffsetDateTime::parse(&row.captured_at, &Rfc3339)
        .map_err(|e| StorageError::Parse(format!("captured_at: {e}")))?;

    let source = row
        .source
        .parse::<InboxSource>()
        .map_err(|e| StorageError::Parse(format!("source: {e}")))?;

    let triage_state = row
        .triage_state
        .parse::<TriageState>()
        .map_err(|e| StorageError::Parse(format!("triage_state: {e}")))?;

    // NULL triaged_to maps to None; a non-NULL value wraps the string.
    let triaged_to = row.triaged_to.map(TypedEntityRef);

    Ok(InboxItem {
        id,
        raw_text: row.raw_text,
        captured_by,
        captured_at,
        source,
        triage_state,
        triaged_to,
    })
}

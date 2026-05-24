// inbox_repository.rs — integration tests for the inbox storage layer.
//
// Each test spins up a fresh in-memory SQLite database, applies all migrations,
// and exercises the three repository functions. The in-memory database is
// isolated per test — no shared state, no ordering dependencies between tests
// even when the test runner executes them in parallel.
//
// What these tests verify:
//   • The round-trip invariant: values written via insert_inbox_item must come
//     back unchanged via fetch_inbox_item (every field, not just id).
//   • The "not found" contract: fetching a non-existent ID returns None, not
//     an error.
//   • List ordering: list_recent_inbox_items returns items newest-first.
//   • List limiting: the `limit` parameter caps the result count.
//   • Enum serialisation: all InboxSource variants survive a write→read cycle
//     without the storage layer choking on an unknown string.
//   • Triaged item: the triaged_to field is preserved when non-null.
//
// These tests do not test business logic (that lives in amity-core unit tests)
// or HTTP semantics (that lives in amity-service tests).

use amity_core::ids::{InboxItemId, MemberId};
use amity_core::inbox::{InboxItem, InboxItemBuilder, InboxSource, TriageState, TypedEntityRef};
use amity_storage::connection::open_database;
use amity_storage::inbox::{fetch_inbox_item, insert_inbox_item, list_recent_inbox_items};
// The `datetime!` macro produces a compile-time `OffsetDateTime` from a literal,
// which gives tests deterministic timestamps without system-clock calls.
use time::macros::datetime;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Open a fresh in-memory database with all migrations applied.
///
/// Each test calls this independently so there is no shared mutable state
/// between tests, even when the test runner runs them in parallel.
///
/// The `:memory:` URL creates a new, empty SQLite database every time.
/// `open_database` applies all migrations before returning the pool.
async fn open_test_db() -> sqlx::SqlitePool {
    open_database("sqlite::memory:")
        .await
        .expect("in-memory database should always open")
}

/// The placeholder member UUID inserted by migration 0001.
///
/// Tests use this UUID when `captured_by` is needed, because the database
/// has a foreign-key constraint on `members(id)` and the in-memory database
/// starts with only this one member row (inserted by the migration itself).
fn placeholder_member_id() -> MemberId {
    // Same UUID as the one in 0001_initial.sql — must match exactly.
    MemberId(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-000000000001")
            .expect("hardcoded UUID is valid"),
    )
}

/// Build a minimal valid inbox item with a specific `captured_at` timestamp.
///
/// `text` and `captured_at` are parameterised so tests can create items with
/// predictable ordering and distinguishable content without duplicating the
/// builder call.
fn make_item(text: &str, captured_at: time::OffsetDateTime) -> InboxItem {
    InboxItemBuilder::new()
        .raw_text(text)
        .captured_by(placeholder_member_id())
        .now(captured_at)
        .build()
        // `expect` is acceptable in test helpers — a panic here is a test
        // setup error, not a production invariant violation.
        .expect("test item should be valid")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn insert_and_fetch_round_trip() {
    // An item written to the database should come back with all fields intact.
    // If any field silently changes (e.g. a timezone gets stripped, an enum
    // serialises differently), this test catches it.
    let pool = open_test_db().await;
    let item = make_item("pick up pencils", datetime!(2026-05-25 10:00:00 UTC));

    insert_inbox_item(&pool, &item)
        .await
        .expect("insert should succeed");

    let fetched = fetch_inbox_item(&pool, item.id)
        .await
        .expect("fetch should succeed")
        // `expect` on the Option: if the item isn't found immediately after
        // insert, something is very wrong with the write or the fetch query.
        .expect("item should exist after insert");

    // Verify every field round-trips, not just id.
    assert_eq!(fetched.id, item.id);
    assert_eq!(fetched.raw_text, item.raw_text);
    assert_eq!(fetched.captured_by, item.captured_by);
    // OffsetDateTime comparison: the stored/parsed value should equal the
    // original. RFC 3339 round-trip preserves UTC offset exactly. If this
    // fails, the timestamp is either being truncated or the timezone offset
    // is being dropped during serialisation.
    assert_eq!(fetched.captured_at, item.captured_at);
    assert_eq!(fetched.source, item.source);
    assert_eq!(fetched.triage_state, item.triage_state);
    assert_eq!(fetched.triaged_to, item.triaged_to);
}

#[tokio::test]
async fn fetch_nonexistent_id_returns_none() {
    // "Not found" must return None, not an error. The service layer decides
    // what to do with a missing item (usually a 404 response). If the storage
    // layer returned an error, the service would have to inspect the error to
    // decide whether it's "not found" vs "I/O failure", which is fragile.
    let pool = open_test_db().await;
    // Generate a fresh ID that was never inserted.
    let missing_id = InboxItemId::new();

    let result = fetch_inbox_item(&pool, missing_id)
        .await
        .expect("fetch should not error for a missing ID");

    assert!(result.is_none(), "expected None for a non-existent ID");
}

#[tokio::test]
async fn list_recent_returns_items_newest_first() {
    // Items should be returned in descending captured_at order regardless of
    // insertion order. The recent-list query uses ORDER BY captured_at DESC.
    // This test inserts the older item first to verify ordering isn't just
    // "insertion order" (which would happen to work but isn't the contract).
    let pool = open_test_db().await;

    // Older item: 08:00
    let older = make_item("older item", datetime!(2026-05-25 08:00:00 UTC));
    // Newer item: 10:00 — two hours later
    let newer = make_item("newer item", datetime!(2026-05-25 10:00:00 UTC));

    // Insert older first, then newer — so if the query returns insertion order
    // instead of timestamp order, the test would fail in the expected way.
    insert_inbox_item(&pool, &older)
        .await
        .expect("insert older");
    insert_inbox_item(&pool, &newer)
        .await
        .expect("insert newer");

    let items = list_recent_inbox_items(&pool, 20)
        .await
        .expect("list should succeed");

    assert_eq!(items.len(), 2, "expected exactly two items");
    // Newest must be first — the Today view relies on this ordering.
    assert_eq!(items[0].raw_text, "newer item", "first item must be newest");
    assert_eq!(
        items[1].raw_text, "older item",
        "second item must be oldest"
    );
}

#[tokio::test]
async fn list_recent_respects_limit() {
    // The `limit` parameter must cap the number of returned items, even when
    // more items exist. The service enforces a maximum of 100; the storage
    // layer just applies whatever limit it receives from the caller.
    let pool = open_test_db().await;

    // Insert three items with distinct timestamps so the query ordering is
    // deterministic.
    for i in 0u8..3 {
        // Stagger timestamps by 1 minute each to ensure distinct ordering.
        let ts = datetime!(2026-05-25 10:00:00 UTC) + time::Duration::seconds(i64::from(i) * 60);
        let item = make_item(&format!("item {i}"), ts);
        insert_inbox_item(&pool, &item).await.expect("insert");
    }

    // Request only 2, even though 3 exist.
    let items = list_recent_inbox_items(&pool, 2)
        .await
        .expect("list should succeed");

    // The storage layer must honour the limit — returning all rows regardless
    // of the limit would be a data leak in some contexts.
    assert_eq!(items.len(), 2, "limit should cap at 2");
}

#[tokio::test]
async fn list_recent_with_empty_database_returns_empty_vec() {
    // An empty inbox must return an empty Vec, not an error. This is the
    // expected state on first run and the correct representation of "no data".
    let pool = open_test_db().await;

    let items = list_recent_inbox_items(&pool, 20)
        .await
        .expect("list should succeed on empty db");

    assert!(items.is_empty(), "expected empty vec for empty database");
}

#[tokio::test]
async fn all_inbox_sources_round_trip_through_storage() {
    // Every InboxSource variant must survive a write→read cycle. If the TEXT
    // representation in `InboxSource::to_string` doesn't match the one in
    // `InboxSource::from_str`, the storage layer will fail to parse the row.
    let pool = open_test_db().await;
    let base_time = datetime!(2026-05-25 10:00:00 UTC);

    let sources = [
        InboxSource::Touch,
        InboxSource::Mobile,
        InboxSource::Share,
        InboxSource::ForwardEmail,
        InboxSource::Voice,
    ];

    for (i, source) in sources.iter().enumerate() {
        // Stagger timestamps by 1 minute so each item has a unique captured_at.
        // Without this, items could be returned in arbitrary order by the list query.
        // `i` is bounded by the sources array length (5); the cast to i64
        // cannot overflow. The lint is suppressed because the range is known.
        #[allow(clippy::cast_possible_wrap)]
        let ts = base_time + time::Duration::seconds(i as i64 * 60);
        let item = InboxItemBuilder::new()
            .raw_text(format!("source test {i}"))
            .captured_by(placeholder_member_id())
            .now(ts)
            .source(*source)
            .build()
            // `unwrap` is acceptable in tests — a failure here means the test
            // data is invalid, which is a setup error.
            .unwrap();

        insert_inbox_item(&pool, &item).await.expect("insert");

        let fetched = fetch_inbox_item(&pool, item.id)
            .await
            .expect("fetch")
            .expect("exists");

        assert_eq!(
            fetched.source, *source,
            "source round-trip failed for {source:?}"
        );
    }
}

#[tokio::test]
async fn triaged_item_stores_typed_entity_ref() {
    // When an item has been triaged (triage_state = Typed, triaged_to = Some(...)),
    // the reference must round-trip through storage intact. If `triaged_to` is
    // mistakenly stored as NULL or truncated, the triage link is lost.
    let pool = open_test_db().await;
    let ts = datetime!(2026-05-25 10:00:00 UTC);

    // Build a base item, then override the triage fields. The builder always
    // sets Untriaged; actual triage transitions happen in the service layer
    // after capture, so we simulate that here by mutating after construction.
    let mut item = make_item("buy printer ink", ts);
    // Set the item as having been triaged to a Task entity.
    item.triage_state = TriageState::Typed;
    item.triaged_to = Some(TypedEntityRef::new(
        "task",
        uuid::Uuid::parse_str("018f1a2b-0000-7000-8000-000000000002").unwrap(),
    ));

    insert_inbox_item(&pool, &item).await.expect("insert");

    let fetched = fetch_inbox_item(&pool, item.id)
        .await
        .expect("fetch")
        .expect("exists");

    // Both the state and the reference must round-trip. A triage_state of
    // Typed with a None triaged_to would be an inconsistent state.
    assert_eq!(fetched.triage_state, TriageState::Typed);
    assert_eq!(fetched.triaged_to, item.triaged_to);
}

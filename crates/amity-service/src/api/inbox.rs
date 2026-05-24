// api/inbox.rs — HTTP handlers for the Inbox API.
//
// Two endpoints:
//   POST /api/v1/inbox
//     Accepts: { "raw_text": "...", "source": "touch" }
//     Returns: the created InboxItem as JSON.
//
//   GET /api/v1/inbox/recent?limit=N
//     Returns: array of the N most recent InboxItems, newest first.
//     Default limit: 20. Maximum: 100.
//
// Both handlers delegate all domain logic to amity-core and all persistence
// to amity-storage. The handlers themselves only: parse the request, call the
// right functions, and serialise the response.
//
// The placeholder member ID (from migration 0001) is used for `captured_by`
// until the Member entity is implemented. This is a documented shortcut.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use amity_core::ids::MemberId;
use amity_core::inbox::{InboxItemBuilder, InboxSource};
use amity_storage::inbox::{insert_inbox_item, list_recent_inbox_items};

use crate::AppState;

// ─── Request / response types ─────────────────────────────────────────────────

/// Request body for `POST /api/v1/inbox`.
///
/// Only two fields are required at capture time — the system supplies
/// `id`, `captured_by`, `captured_at`, and `triage_state` automatically.
#[derive(Debug, Deserialize)]
pub struct CaptureInboxItemRequest {
    /// The raw text of the captured thought. Must not be blank.
    pub raw_text: String,

    /// How the item was captured. Defaults to `touch` if absent.
    #[serde(default = "default_source")]
    pub source: InboxSource,
}

fn default_source() -> InboxSource {
    // Hub touch is the default capture path.
    InboxSource::Touch
}

/// Query parameters for `GET /api/v1/inbox/recent`.
#[derive(Debug, Deserialize)]
pub struct ListRecentQuery {
    /// Maximum number of items to return. Capped at 100; defaults to 20.
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    20
}

// ─── Shared response type ─────────────────────────────────────────────────────

/// JSON representation of an inbox item, used in both the create and list
/// responses. This is a flat serialisation of `InboxItem` suitable for the
/// API surface; it does not expose internal storage details.
#[derive(Debug, Serialize)]
pub struct InboxItemResponse {
    pub id: String,
    pub raw_text: String,
    pub captured_by: String,
    /// RFC 3339 timestamp, e.g. `"2026-05-25T10:00:00Z"`.
    pub captured_at: String,
    pub source: String,
    pub triage_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triaged_to: Option<String>,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

/// `POST /api/v1/inbox` — capture a new inbox item.
///
/// Generates a fresh ID and timestamp, inserts the item into the database,
/// and returns the created item as JSON with HTTP 201 Created.
///
/// Returns HTTP 422 if `raw_text` is blank.
/// Returns HTTP 500 on unexpected storage errors.
///
/// # Panics
///
/// Panics if the hardcoded placeholder member UUID is not valid. This cannot
/// happen in practice because the UUID is a compile-time constant; the panic
/// is documented here to satisfy the pedantic lint.
pub async fn capture_inbox_item(
    State(state): State<AppState>,
    Json(req): Json<CaptureInboxItemRequest>,
) -> impl IntoResponse {
    // Use the system clock for `now`. The clock is not injected in the handler
    // because handlers are async axum functions with fixed signatures; tests
    // that need clock control should test the domain logic directly.
    let now = OffsetDateTime::now_utc();

    // Use the placeholder member ID until the Member entity is implemented.
    // See migration 0001_initial.sql for the rationale.
    let placeholder_member = MemberId(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-000000000001")
            .expect("hardcoded UUID is always valid"),
    );

    let item = match InboxItemBuilder::new()
        .raw_text(req.raw_text)
        .captured_by(placeholder_member)
        .now(now)
        .source(req.source)
        .build()
    {
        Ok(item) => item,
        Err(amity_core::inbox::InboxError::EmptyText) => {
            // 422 Unprocessable Entity — the request was well-formed JSON but
            // the business rule (non-empty text) was violated.
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "raw_text must not be empty" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "unexpected error building inbox item");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = insert_inbox_item(&state.db, &item).await {
        tracing::error!(error = %e, "failed to insert inbox item");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let response = inbox_item_to_response(&item);

    // 201 Created with the full item in the body so the client can display it
    // immediately without a separate fetch.
    (StatusCode::CREATED, Json(response)).into_response()
}

/// `GET /api/v1/inbox/recent?limit=N` — list recent inbox items.
///
/// Returns HTTP 200 with a JSON array of items, newest first.
/// Returns HTTP 400 if `limit` exceeds 100.
pub async fn list_recent(
    State(state): State<AppState>,
    Query(params): Query<ListRecentQuery>,
) -> impl IntoResponse {
    // Enforce the maximum limit at the API boundary. The storage layer accepts
    // any u32; we cap it here so the API contract is clear and tested.
    if params.limit > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "limit must be ≤ 100" })),
        )
            .into_response();
    }

    match list_recent_inbox_items(&state.db, params.limit).await {
        Ok(items) => {
            let responses: Vec<InboxItemResponse> =
                items.iter().map(inbox_item_to_response).collect();
            Json(responses).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to list inbox items");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ─── Private helpers ─────────────────────────────────────────────────────────

/// Convert a domain `InboxItem` into its JSON response representation.
///
/// Uses `Display` implementations for enum variants (which produce the same
/// `snake_case` strings stored in the database), so the API and storage wire
/// formats stay consistent.
fn inbox_item_to_response(item: &amity_core::inbox::InboxItem) -> InboxItemResponse {
    // RFC 3339 timestamp format — unambiguous, sortable, human-readable.
    let captured_at = item
        .captured_at
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| item.captured_at.to_string());

    InboxItemResponse {
        id: item.id.to_string(),
        raw_text: item.raw_text.clone(),
        captured_by: item.captured_by.to_string(),
        captured_at,
        source: item.source.to_string(),
        triage_state: item.triage_state.to_string(),
        triaged_to: item.triaged_to.as_ref().map(|r| r.0.clone()),
    }
}

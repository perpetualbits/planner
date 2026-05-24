// lib.rs — Tauri application library.
//
// Defines the Tauri commands exposed to the web frontend and sets up the
// Tauri application builder.
//
// Commands:
//   capture_inbox_item  — POST /api/v1/inbox, returns the created item as JSON.
//   list_recent_inbox   — GET /api/v1/inbox/recent?limit=N, returns array of items.
//
// Both commands delegate to amity-service over HTTP on localhost. The service
// address is hardcoded to http://127.0.0.1:7890 for the prototype; a later
// task will read it from the Tauri app config.

use serde::{Deserialize, Serialize};

// ─── Service address ──────────────────────────────────────────────────────────

/// Base URL of the amity-service instance this application communicates with.
///
/// Hardcoded for the prototype. A later task will read this from the Tauri
/// app config or from a sidecar-managed port.
const SERVICE_BASE_URL: &str = "http://127.0.0.1:7890";

// ─── Shared data types ────────────────────────────────────────────────────────

/// An inbox item as returned by the service API.
///
/// Mirrors `InboxItemResponse` in amity-service. Kept as a separate type so
/// the frontend-facing shape can evolve independently of the service type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: String,
    pub raw_text: String,
    pub captured_by: String,
    pub captured_at: String,
    pub source: String,
    pub triage_state: String,
    pub triaged_to: Option<String>,
}

/// Request body for capturing a new inbox item.
#[derive(Debug, Serialize)]
struct CaptureRequest {
    raw_text: String,
    source: String,
}

// ─── Tauri commands ───────────────────────────────────────────────────────────

/// Capture a new inbox item by forwarding the request to amity-service.
///
/// Called from the frontend when the user submits the capture form.
/// Returns the created `InboxItem` on success, or an error string that the
/// frontend can display.
///
/// # Errors
///
/// Returns a string error if the HTTP request fails or the service returns
/// a non-2xx status. The frontend displays this as a plain message; no
/// toast notifications, no animated error states — consistent with the calm
/// aesthetic.
#[tauri::command]
pub async fn capture_inbox_item(raw_text: String) -> Result<InboxItem, String> {
    let client = reqwest::Client::new();

    let body = CaptureRequest {
        raw_text,
        source: "touch".to_owned(),
    };

    let response = client
        .post(format!("{SERVICE_BASE_URL}/api/v1/inbox"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("failed to reach amity-service: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("service error {status}: {body}"));
    }

    response
        .json::<InboxItem>()
        .await
        .map_err(|e| format!("failed to parse response: {e}"))
}

/// Fetch the most recent inbox items from amity-service.
///
/// Called from the frontend on mount and after each successful capture.
/// Returns a Vec of `InboxItem`, newest first.
///
/// # Errors
///
/// Returns a string error if the HTTP request fails.
#[tauri::command]
pub async fn list_recent_inbox(limit: u32) -> Result<Vec<InboxItem>, String> {
    let client = reqwest::Client::new();

    // Cap at 100 to match the service's own maximum, even though the service
    // would reject a higher limit itself. Belt-and-suspenders.
    let effective_limit = limit.min(100);

    let response = client
        .get(format!(
            "{SERVICE_BASE_URL}/api/v1/inbox/recent?limit={effective_limit}"
        ))
        .send()
        .await
        .map_err(|e| format!("failed to reach amity-service: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(format!("service error {status}"));
    }

    response
        .json::<Vec<InboxItem>>()
        .await
        .map_err(|e| format!("failed to parse response: {e}"))
}

// ─── Application entry ────────────────────────────────────────────────────────

/// Build and run the Tauri application.
///
/// Called from `main.rs`. Registers all Tauri commands so the frontend can
/// invoke them via `invoke(...)`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            capture_inbox_item,
            list_recent_inbox,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

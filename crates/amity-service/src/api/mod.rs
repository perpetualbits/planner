// api/mod.rs — HTTP API module.
//
// This module owns the axum router and all handler modules.
// Each entity gets its own handler module; the router here wires them together.
//
// All routes are under `/api/v1/` so that a future breaking change can be
// deployed alongside the current version without path conflicts.
//
// Current routes:
//   POST /api/v1/inbox         — capture a new inbox item
//   GET  /api/v1/inbox/recent  — list recent inbox items

pub mod inbox;

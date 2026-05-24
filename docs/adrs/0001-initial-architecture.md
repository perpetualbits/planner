# ADR-0001 — Initial workspace layout and technology choices

**Date:** 2026-05-25
**Status:** Accepted

---

## Context

Task 1 establishes the repository structure and the first end-to-end feature
(the Inbox entity). Before writing code, the workspace layout, backend
framework, database, and frontend framework must be chosen. These choices carry
forward to every subsequent entity, so getting them right — and recording why —
is worth the ceremony.

The project's guiding constraints, from the philosophy and brief:

- Local-first by architecture. No external services in the data path.
- The home-node trajectory (brief §17) requires the architecture to migrate
  cleanly from "Android tablet in kiosk mode" to "small home server + separate
  display" without a rewrite.
- Separation of concerns: the frontend should talk to a service, not directly
  to the database. This keeps the data layer testable and keeps the eventual
  mobile companion app on the same footing as the hub.

---

## Decision

### Cargo workspace layout

A single Cargo workspace at the repository root with this crate structure:

```
crates/
  amity-core      — domain types, no I/O
  amity-storage   — SQLite persistence via sqlx
  amity-service   — axum HTTP service
apps/
  hub-tauri       — Tauri 2 desktop shell
```

**Why this layout:**
The strict one-way dependency graph (`service → storage → core`) enforces the
separation of concerns the project requires. `amity-core` has no I/O
dependencies, which means domain logic is testable without a database or
network. The `apps/` directory is separate from `crates/` to signal that it is
an end-user artifact rather than a library.

**Alternatives considered:**
- A monolithic single crate: rejected because it collapses the enforced
  separation. A change to the HTTP routing layer would rebuild the domain types.
- A microservices layout (one binary per domain area): rejected as premature.
  The household data model is a single coherent domain; the benefit of separate
  binaries would only emerge at a scale this project is unlikely to reach.

### Backend: axum + sqlx + SQLite

- **axum** for HTTP: built on tokio and tower; clean middleware composition;
  idiomatic for modern async Rust. The alternative was actix-web; axum was
  chosen because its tower integration is superior and its extract-based handler
  design is easier to test.
- **sqlx** for database access: compile-time checked queries (via the offline
  cache, to be set up in a follow-up task); supports both SQLite and Postgres
  from the same migration; no ORM magic.
- **SQLite** as the storage engine for MVP: zero-infrastructure, single-file,
  appropriate for a household device. The schema is designed for later Postgres
  migration (TEXT UUIDs, ISO-8601 datetimes, no SQLite-specific pragmas in
  queries).
- **UUID v7** for entity IDs: time-ordered, globally unique, and index-friendly.
  The ordering property means related rows stay physically close in the B-tree,
  which matters for the household-scale dataset that will accumulate over years.

### Frontend: SolidJS inside Tauri 2

- **Tauri 2** for the desktop shell: gives a native window and a Rust backend
  with minimal overhead. Tauri commands bridge between the web frontend and the
  Rust service layer cleanly. Tauri's security model (CSP, command allowlist) is
  appropriate for a household device.
- **SolidJS** for the web frontend: small bundle, fine-grained reactivity, low
  ceremony. The alternatives considered were plain HTML/JS (rejected: manual DOM
  manipulation becomes unwieldy as soon as the item list has filtering or
  ordering) and React (rejected: bundle size contradicts the calm-minimal
  posture; the React ecosystem brings dependencies the project doesn't need).
  Svelte was also a candidate; SolidJS was preferred because its reactivity
  model is explicit and its runtime is smaller.
- **Vite** as the build tool: standard for SolidJS; fast hot-reload.

### Configuration

- Service config: TOML at `$XDG_CONFIG_HOME/amity/config.toml` (platform
  equivalent on macOS/Windows via the `directories` crate). Falls back to
  built-in defaults if absent. Config file absent is not an error; the service
  is runnable out of the box.

---

## Consequences

**Good:**
- The strict dependency graph enforces the separation of concerns at the
  compiler level. Violations are compile errors, not code review failures.
- `amity-core` unit tests run instantly (no I/O setup).
- The service integration tests spin up an in-memory SQLite database; no
  external infrastructure required in CI.
- The Tauri + SolidJS stack produces a small, fast frontend with native window
  integration.

**Accepted trade-offs:**
- The Tauri frontend talks to `amity-service` over localhost HTTP rather than
  via a Tauri sidecar command. This adds a round-trip but keeps the service
  independently addressable (the mobile companion app uses the same API).
- sqlx compile-time query checking requires an offline cache (`cargo sqlx prepare`).
  This is not set up in Task 1; queries use runtime checking in the initial
  implementation. A follow-up task will generate the cache and switch to checked
  macros.
- SolidJS requires a build step (Vite). The alternative (plain JS) would avoid
  this but would make the component model and reactivity manual. The trade-off
  is acceptable.

---

*This ADR supersedes any informal decisions made before Task 1. Future
architectural changes get their own ADRs, referencing this one where relevant.*

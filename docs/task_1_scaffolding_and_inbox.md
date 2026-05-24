# Task 1 — Repository scaffolding and Inbox entity end-to-end

*Claude Code task description. Read in full before starting; surface questions before writing code.*

---

## Context

This is the first implementation task for the Amity project. Before writing any code, read:

- `docs/amity_philosophy.md` (the values; load-bearing for every decision)
- `docs/amity_brief.md` sections 1–6 (the design and the data model)
- `docs/coding_guidelines.md` (general posture, comment density requirement)
- `docs/rust_guidelines.md` (crate choices, workspace layout, idiomatic patterns)
- `docs/claude_code_workflow.md` (how this task fits the project's working pattern)

This task does two things at once:

1. **Stand up the project structure** — Cargo workspace, the documented crate layout, CI tooling, pre-commit hooks, the basic development environment.
2. **Implement the `InboxItem` entity end-to-end** — from the data model type in `amity-core` through storage, service, and a minimal Tauri frontend that can capture and display inbox items.

Done correctly, the deliverable is a runnable prototype: launch the hub app, tap a button, type "test note", see it stored, see it appear in a list. Every architectural seam touched at least once. The patterns established here will be reused for every subsequent entity.

## Why the Inbox first

The Inbox is the conceptual foundation of Amity (see brief section 6.3). Capture is the system's first promise — if capture doesn't work, nothing else matters. Starting here means the foundation gets the most attention, and the patterns we establish for it carry through the rest of the codebase.

The Inbox is also small enough to fit one task: a single entity with a small number of fields, a single primary operation (capture), and a single view (list of recent items). Real complexity (typed entities, the surfacing layer, member switching, voice) lands later.

## Deliverables

### Repository structure

A Cargo workspace at the repository root, organised per `rust_guidelines.md`:

```
amity/
├── Cargo.toml                          # workspace manifest
├── rust-toolchain.toml                 # pinned stable Rust
├── .gitignore
├── .editorconfig
├── README.md                           # short orientation
├── LICENSE                             # AGPL-3.0
├── CONTRIBUTING.md                     # DCO sign-off instructions
├── docs/                               # design documents (already in place)
├── scripts/
│   └── comment-density.sh              # audit comment density
├── crates/
│   ├── amity-core/                     # domain types, no I/O
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── ids.rs                  # typed ID newtypes
│   │   │   └── inbox.rs                # InboxItem type
│   │   └── tests/                      # unit tests live in src/ under #[cfg(test)]
│   ├── amity-storage/                  # persistence
│   │   ├── Cargo.toml
│   │   ├── migrations/
│   │   │   └── 0001_initial.sql        # creates inbox_items table
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── connection.rs           # pool setup
│   │   │   └── inbox.rs                # inbox repository functions
│   │   └── tests/
│   │       └── inbox_repository.rs     # integration test against test SQLite
│   └── amity-service/                  # HTTP service
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                  # axum app construction
│           ├── main.rs                 # binary entrypoint
│           ├── api/
│           │   ├── mod.rs
│           │   └── inbox.rs            # POST /inbox, GET /inbox/recent
│           └── config.rs               # service config loading
└── apps/
    └── hub-tauri/                      # Tauri shell (frontend framework TBD — see Open Questions)
        ├── src-tauri/
        │   ├── Cargo.toml
        │   ├── tauri.conf.json
        │   └── src/
        │       └── main.rs
        └── src/                        # web frontend
            └── ...                     # see Open Questions about framework choice
```

CI workflow (`.github/workflows/ci.yml`) runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -W clippy::pedantic`, `cargo test --workspace`, and `cargo doc --workspace --no-deps`.

Pre-commit hook (managed by [`pre-commit`](https://pre-commit.com/)) runs `cargo fmt` and the comment-density check on staged files.

### `amity-core::InboxItem`

The domain type, per brief section 6.3:

```rust
/// An untriaged item captured into the inbox.
///
/// The inbox is the universal capture mechanism — the first promise the
/// system makes. Items here have no required structure beyond their raw
/// text; triage into a typed entity (Task, Event, etc.) is optional and
/// deferrable, per the design in §6.3 of the brief.
///
/// See `docs/amity_brief.md` section 6.3 for the conceptual context.
pub struct InboxItem {
    pub id: InboxItemId,
    pub raw_text: String,
    pub captured_by: MemberId,
    pub captured_at: OffsetDateTime,
    pub source: InboxSource,
    pub triage_state: TriageState,
    pub triaged_to: Option<TypedEntityRef>,
    // attachments[] deferred — adds a complete second concern; comes in a follow-up task
}
```

Includes:
- `InboxItemId(Uuid)` newtype in `amity-core::ids`.
- `MemberId(Uuid)` newtype (the member entity itself comes in a later task; the ID type can exist now).
- `InboxSource` enum (`Voice`, `Touch`, `Mobile`, `Share`, `ForwardEmail`).
- `TriageState` enum (`Untriaged`, `Typed`, `Dismissed`, `KeptAsNote`).
- `TypedEntityRef` placeholder (just a String for now containing "entity_type:uuid" — proper typed refs come when those entities exist).

The type and its enums get serde derives, `Debug`, `Clone`, `PartialEq` where sensible. Doc comments on every public item per the guidelines. Unit tests covering construction invariants (e.g. `raw_text` non-empty, `captured_at` not in the future relative to a passed-in `now`).

### `amity-storage::inbox`

A repository module exposing:

```rust
pub async fn insert_inbox_item(pool: &SqlitePool, item: &InboxItem) -> Result<(), StorageError>;
pub async fn fetch_inbox_item(pool: &SqlitePool, id: InboxItemId) -> Result<Option<InboxItem>, StorageError>;
pub async fn list_recent_inbox_items(pool: &SqlitePool, limit: u32) -> Result<Vec<InboxItem>, StorageError>;
```

Plus the migration `0001_initial.sql` creating the `inbox_items` table. Schema design constraints:

- Use ISO-8601 strings for datetimes in SQLite (sqlx handles this transparently with the `time` feature).
- Use TEXT for UUIDs (sqlx convention; portable to Postgres later).
- Index on `captured_at DESC` for the recent-list query.
- Foreign key to `members(id)` for `captured_by` — but the `members` table doesn't exist yet. Solution: include a `members` table in this migration with just an `id` column for now; member entity proper comes in a later task. Insert a single placeholder member at migration time with a known UUID so the prototype works without a member-management feature yet.

Integration test in `crates/amity-storage/tests/inbox_repository.rs`: spin up an in-memory or temp-file SQLite, apply migrations, insert items, query them back, assert correctness. Covers the three repository functions plus at least one edge case (empty list, fetching a non-existent ID).

### `amity-service::api::inbox`

Two axum handlers:

- `POST /api/v1/inbox` — accepts JSON `{ "raw_text": "...", "source": "touch" }`, generates a fresh ID and timestamp, inserts via storage, returns the created item as JSON. Use the placeholder member ID for `captured_by` for now.
- `GET /api/v1/inbox/recent?limit=N` — returns the N most recent inbox items as JSON. Default limit if absent: 20. Maximum: 100.

The service binary in `main.rs` reads a config file (`config.toml`) for the database URL, server bind address, and listen port; falls back to sensible defaults if the config is absent. Uses `tracing-subscriber` to set up structured logging.

Integration test in `crates/amity-service/tests/inbox_api.rs`: spin up the service against a test database, hit both endpoints with `reqwest`, assert responses.

### `apps/hub-tauri`

A minimal Tauri 2 application that:

- Renders a single screen with two areas: a "capture" form (a text input + a submit button) and a "recent items" list.
- The capture form, on submit, calls a Tauri command which calls the local Amity service's `POST /api/v1/inbox`. Clears the input on success.
- The recent items list calls `GET /api/v1/inbox/recent?limit=20` on mount and after each successful capture.
- Uses Atkinson Hyperlegible font (loaded as a project asset).
- 60×60 minimum touch targets on the submit button.
- Plain calm visual style — no decorative elements, no "submit" animation, no toast notifications. Capture happens; the item appears in the list; that's the entire feedback. (This is the empty-state philosophy applied to interaction design.)

This is *not* the full hub-at-rest design (no clock, no weather, no status patch) — that comes in a later task. This is the smallest UI that exercises the architecture.

### Documentation

- `README.md`: 1–2 paragraphs orienting a new contributor, link to the philosophy and brief.
- `CONTRIBUTING.md`: DCO sign-off, commit format, how to run the test suite.
- An ADR `docs/adrs/0001-initial-architecture.md` recording the workspace layout decision and why.

## Open Questions

Surface these before committing code; the maintainer will decide.

### Web frontend framework inside Tauri

The Tauri shell needs a web frontend. Options worth raising:

- **Plain HTML + vanilla JS** — minimum framework dependency, maximum honesty about what the UI does. Build tooling: vite for hot reload only.
- **Svelte** — small bundle, reactive, low ceremony. Good fit for the calm aesthetic.
- **Solid** — smaller and faster than React; mature enough for production.
- **React** — most ecosystem support but heavier and more idiomatic patterns to enforce.

For a prototype I'd lean toward plain HTML/JS or Svelte. React is overkill at this scope and brings a JS bundle that contradicts the calm-and-minimal posture. But this is a real choice and worth a quick maintainer call before committing.

### Comment density script

The `scripts/comment-density.sh` referenced in the coding guidelines doesn't exist yet. It needs to count comment lines and code lines per file and report the ratio. Should I write it as part of this task, or defer to a separate task? I'd suggest including a minimal version here (bash + simple grep) so the CI check has something to call.

### Configuration format and location

Per the Rust guidelines, configuration is TOML. Where does the file live for the service? Suggested default: `$XDG_CONFIG_HOME/amity/config.toml` on Linux, sensible equivalents on other platforms (`directories` crate handles this). For the prototype, a `config.toml` in the working directory is simpler and acceptable.

### Member ID for the prototype

This task introduces a placeholder member (a single hardcoded UUID inserted at migration time) so the prototype can capture inbox items without needing a member-management feature. This is a deliberate shortcut and should be documented in the migration's comments. The proper Member entity, with creation/management and the two-tier governance, lands in a later task. Confirm this shortcut is acceptable; if not, member management has to come first and this task gets larger.

## Acceptance criteria

A reviewer (human or Claude) should be able to verify:

- [ ] `cargo build --workspace` succeeds.
- [ ] `cargo test --workspace` passes; tests include the integration tests called out above.
- [ ] `cargo clippy --workspace --all-targets -- -W clippy::pedantic` passes (warnings reviewed in the PR description; suppressions justified).
- [ ] `cargo fmt --check` passes.
- [ ] `cargo doc --workspace --no-deps --all-features` builds clean (no missing doc warnings on public items).
- [ ] The comment density check passes for every new Rust file (target 50% as in the guidelines).
- [ ] `cargo run --bin amity-service` starts the service against a fresh database; migrations apply automatically; service is reachable at the configured address.
- [ ] The Tauri app builds and runs (`cd apps/hub-tauri && cargo tauri dev`).
- [ ] Manually exercising the Tauri app: a typed inbox item appears in the recent list after submission.
- [ ] The ADR `0001-initial-architecture.md` exists and is non-trivial.
- [ ] All commits are DCO-signed (`git commit -s`) and follow Conventional Commits.

## Scope guardrails

This task does **not** include:

- Member management (creation, switching, PINs/NFC). Placeholder member only.
- Other entities (Task, Event, Meal, etc.). Inbox only.
- The hub-at-rest UI (clock, weather, status patch). Minimal capture+list only.
- Voice capture. Touch only.
- Mobile companion app. Hub only.
- Authentication on the API. The service binds to localhost only as a basic isolation; proper auth comes when members come.
- ICS calendar aggregation. Out of scope by a wide margin.
- Postgres compatibility. SQLite only for this task; Postgres support comes when there is a reason to test it.
- The empty state, the surfacing layer, the Today view. All later.

If the work-in-progress is creeping outside these guardrails, stop and ask. The discipline of staying small is what makes the first task valuable.

## Reading order suggestion

In order:

1. The philosophy document (orient on values).
2. Section 6 of the brief (the data model, especially 6.3 on the Inbox).
3. The Rust guidelines (idioms, crate choices, workspace layout).
4. This task description.
5. The coding guidelines (comment density posture).
6. The Claude Code workflow document (the working pattern with the maintainer).

Then plan the approach, confirm with the maintainer, then begin implementation.

---

*Estimated effort: 3–5 focused days. If the work is exceeding 5 days substantially, the scope has expanded — flag it.*

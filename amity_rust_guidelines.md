# Amity — Rust Guidelines

*Rust-specific guidance, layered on top of the general coding guidelines. Covers crate choices, idiomatic patterns, error handling, async, and the data layer.*

---

## Edition and toolchain

- **Rust edition 2024** (or the latest stable when the project is initialised).
- **Toolchain pinned in `rust-toolchain.toml`** at the workspace root. Major version bumps are deliberate, not implicit.
- **`cargo clippy -- -W clippy::pedantic`** runs in CI. Pedantic lints are reviewed case-by-case; not every warning needs to be fixed, but each is considered.
- **`cargo fmt`** is enforced; pre-commit hook runs it automatically.

## Workspace structure

The project is a Cargo workspace. Crate layout:

```
amity/
├── Cargo.toml                  # workspace
├── crates/
│   ├── amity-core/             # domain entities, business logic, no I/O
│   ├── amity-storage/          # persistence layer; sqlx repositories
│   ├── amity-service/          # the actual application service, HTTP API
│   ├── amity-ical/             # ICS calendar aggregation
│   ├── amity-voice/            # transcription + intent parsing
│   └── amity-cli/              # admin CLI for hub operations
└── apps/
    └── hub-tauri/              # Tauri desktop/mobile shell
```

Rules:

- **`amity-core` has no I/O dependencies.** No tokio, no sqlx, no reqwest. Pure domain logic. Testable without infrastructure.
- **Each crate has a clear single responsibility.** If a crate's name needs an "and" to describe its purpose, it should be split.
- **Inter-crate dependencies are explicit and one-way.** `amity-service` depends on `amity-core` and `amity-storage`, never the reverse.

## Preferred crates

These are the chosen dependencies for the listed concerns. Use them rather than alternatives unless there is a specific, documented reason (which goes in an ADR).

### Runtime and async

- **`tokio`** (multi-thread runtime) for async. The default and the only realistic choice.
- **`tokio-util`** for codecs, sync helpers, and shutdown coordination.

### Serialisation

- **`serde`** + **`serde_json`** for JSON (API surface, configuration).
- **`toml`** for human-edited configuration files.
- **`serde_yaml`** is acceptable for ADR front matter and similar; not preferred for runtime config.

### Error handling

- **`thiserror`** for error types in library crates (`amity-core`, `amity-storage`, etc.). Each crate exposes its own typed errors.
- **`anyhow`** for error handling in application code (`amity-service`, `amity-cli`). The boundary between typed and erased errors is at the crate boundary; library code never returns `anyhow::Error`.
- **No `Box<dyn Error>` in public APIs.** Always typed.

### Logging and tracing

- **`tracing`** + **`tracing-subscriber`** for structured logging. Spans for request lifecycles, structured fields for context.
- **`tracing-appender`** for log file rotation on the home node.
- **No `log` or `env_logger`.** `tracing` is the chosen path; mixing is confusing.

### HTTP and web

- **`axum`** for the HTTP service. Built on `tower` and `tokio`; clean middleware composition; idiomatic.
- **`tower`** for middleware abstractions (timeouts, tracing, rate limits).
- **`reqwest`** (with `rustls` features, not OpenSSL) for outbound HTTP — calendar ICS fetches, weather queries.

### Database

- **`sqlx`** with both `sqlite` and `postgres` features. Schema designs are portable between the two. Compile-time-checked queries via `query!`/`query_as!` macros.
- **Migrations** via sqlx's built-in migration machinery (`sqlx::migrate!`). Migrations are in `crates/amity-storage/migrations/`.
- **No ORM.** Repositories are explicit functions in `amity-storage`. The data layer is a small focused surface, not a magic system.

### IDs

- **`uuid` with `v7` feature.** Time-ordered UUIDs for entity IDs — index locality plus global uniqueness.
- IDs are typed: `TaskId(Uuid)`, `EventId(Uuid)`, not bare `Uuid` in public APIs. Prevents accidental misuse.

### Time

- **`time` crate (`time` 0.3+)** for date and time handling. Strictly preferred over `chrono` for new code — `time` has cleaner ergonomics, better timezone handling, and is more honest about ambiguity.
- **`time-tz`** for IANA timezone database integration.
- Wall-clock times anchored to a timezone, not stored as naive datetimes.

### CLI

- **`clap`** with derive macros for argument parsing.

### Testing

- **`tokio-test`** for testing async code.
- **`insta`** for snapshot tests where they earn their keep (rendered output, complex structures).
- **`proptest`** for property tests on domain logic (recurrence rules, time arithmetic, fairness invariants).

### Voice (post-MVP)

- **`whisper-rs`** (whisper.cpp bindings) for on-device transcription, when voice arrives.

## Idiomatic patterns

### Builder pattern for complex constructors

Domain entities with many fields use builders. The builder validates invariants before constructing the entity. The entity itself is immutable after construction; updates go through repository methods.

### Newtype wrapping

Domain IDs and quantities are newtype-wrapped: `TaskId(Uuid)`, `Effort(u8)`, `Servings(u8)`. This prevents whole classes of "I passed the wrong number to the function" bugs at compile time.

### `From` and `TryFrom`

Conversions between types use the standard traits. `From` for infallible conversions, `TryFrom` for fallible ones with a typed error. Custom `parse`/`convert` functions are a smell.

### `impl Trait` in return position

Prefer `impl Iterator<Item = Task>` over `Vec<Task>` when returning a sequence — defer allocation to the caller. Use `Vec<T>` only when the collection itself is the meaningful thing.

### Async function composition

Async functions return `impl Future<...>` types in trait definitions (Rust 2024 supports this directly via `async fn` in traits). No `async-trait` macro unless needed for object safety.

### Pattern matching, not `if let` chains

Pattern matching is exhaustive and clear. `match` over deeply nested `if let`s. The compiler's exhaustiveness check is a feature, not a hassle.

## Error handling specifics

### Typed error definitions

```rust
#[derive(Debug, thiserror::Error)]
pub enum PantryError {
    #[error("pantry item not found: {0}")]
    NotFound(PantryItemId),

    #[error("invalid pantry level transition: {from:?} -> {to:?}")]
    InvalidTransition { from: PantryLevel, to: PantryLevel },

    #[error("database error")]
    Database(#[from] sqlx::Error),
}
```

The `#[from]` attribute lets the error propagate cleanly with `?`. Errors carry context; they are never just strings.

### Never panic on user-provided input

User-provided input (voice transcriptions, forwarded emails, ICS feeds) is parsed defensively. Failures return errors, not panics. The boundary between "we control this" and "we don't" is where parsing happens, and that boundary is comment-dense and well-tested.

## Concurrency rules

- **Sharing state across tasks: `Arc<T>` for shared immutable, `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for shared mutable.** Prefer immutable where possible.
- **Channels for producer-consumer**: `tokio::sync::mpsc` for one-shot or bounded, `tokio::sync::broadcast` for fan-out.
- **No `std::sync::Mutex` in async code** — use `tokio::sync::Mutex` to avoid blocking the runtime.
- **Cancellation safety**: every `.await` point is a potential cancellation point. Long-running operations honour cancellation; partial state is cleaned up.

## Database patterns

### Repository functions

Each entity has a module in `amity-storage` exposing functions:

```rust
pub async fn fetch_task(
    db: &SqlitePool,
    task_id: TaskId,
) -> Result<Option<Task>, StorageError> {
    // Pure SQL query. No business logic. No silent transformations.
    // Returns Option because "not found" is a valid result, not an error.
    sqlx::query_as!(
        Task,
        r#"SELECT ... FROM tasks WHERE id = ?"#,
        task_id.0,
    )
    .fetch_optional(db)
    .await
    .map_err(StorageError::from)
}
```

Business logic lives in `amity-core`. Storage is dumb on purpose — easy to test, easy to swap, easy to reason about.

### Migrations

- Each migration is a single SQL file with a date-prefix name: `20260615_create_tasks.sql`.
- Migrations are forward-only. Down-migrations are not used; mistakes are corrected with a new forward migration.
- Every migration is tested by being applied to a fresh database in CI.
- Migrations that involve data transformation include comments explaining the transformation's purpose.

### Compile-time checked queries

Use sqlx's `query!`, `query_as!`, and `query_scalar!` macros. The compile-time SQL verification catches schema/query mismatches before runtime. The `prepare` step is a CI requirement.

## Public API surface

### Doc comments

Every public item carries `///` documentation. The doc comment includes:

- A one-line summary.
- A longer paragraph if context is needed.
- A `# Examples` section for non-trivial APIs.
- A `# Errors` section if the function returns `Result`.
- A `# Panics` section if the function can panic (and a justification for why a non-panicking variant doesn't exist).

`cargo doc --no-deps --all-features` builds clean documentation. CI checks for missing doc comments on public items.

### Semver discipline

Once a crate has a stable API surface (1.0+), breaking changes require a major version bump. Pre-1.0 the project moves more freely, but every breaking change is called out in the changelog.

## What not to do

These are anti-patterns specifically called out for Amity:

- **No `panic!` in production code paths.** Even for "this should never happen", use a `Result` with a clear error variant; the compiler enforces handling.
- **No silent `Result` discards.** No `let _ = some_fallible_call().await;` without a comment explaining why the failure is acceptable.
- **No `unsafe` without an ADR.** Unsafe blocks require justification; the project should have zero or near-zero unsafe code.
- **No globals.** No `lazy_static`, no `OnceCell` for configuration, no implicit context. Pass things explicitly.
- **No "convenience" extension traits on third-party types.** They confuse the call site for marginal benefit.
- **No procedural macros written in this project** unless a specific need arises that justifies the maintenance burden. Use existing macros from the ecosystem.

---

*Rust idioms evolve. These guidelines will be updated as the project matures and as the Rust community settles on better patterns. Significant changes go through ADRs.*

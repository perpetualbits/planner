// lib.rs — amity-core public API.
//
// amity-core holds all domain types and business logic with no I/O dependencies.
// Nothing in this crate touches the filesystem, network, or a database — that
// separation makes every module here testable without infrastructure.
//
// Crate dependency order (all arrows go downward; no upward dependencies):
//   amity-service → amity-storage → amity-core
//
// Modules:
//   ids    — typed ID newtypes for all entities (InboxItemId, MemberId, …)
//   inbox  — InboxItem domain type and its builder

/// Typed ID newtypes. See module docs for the rationale.
pub mod ids;

/// `InboxItem`, `InboxSource`, `TriageState`, `TypedEntityRef`, and `InboxItemBuilder`.
///
/// The Inbox is the first promise Amity makes — see brief §6.3.
pub mod inbox;

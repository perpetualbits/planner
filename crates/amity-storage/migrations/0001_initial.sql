-- Migration 0001 — initial schema.
--
-- Creates the tables required for the Inbox feature (Task 1).
-- All subsequent entity tables arrive in later migrations; this migration
-- establishes only what is needed to run the prototype end-to-end.
--
-- Design notes:
--   • UUIDs are stored as TEXT (not BLOB) so that rows are human-readable
--     in the SQLite shell and the format is portable to Postgres without
--     a column-type migration. sqlx handles TEXT↔Uuid transparently.
--   • Datetimes are stored as ISO-8601 TEXT (e.g. "2026-05-25T10:00:00Z").
--     sqlx's `time` feature handles the TEXT↔OffsetDateTime conversion.
--   • STRICT mode is used on every table. It turns SQLite's type-affinity
--     flexibility into genuine type enforcement, catching driver bugs early.
--   • The `members` table is a minimal stub — just enough to satisfy the
--     foreign-key constraint on `inbox_items.captured_by`. The full Member
--     entity (governance, PINs, NFC, per-member settings) arrives in a
--     later task. One placeholder member is inserted at migration time so
--     the prototype works without a member-management feature.

-- Enable foreign-key enforcement. SQLite disables it by default for
-- backwards compatibility; we want the constraint to actually fire.
PRAGMA foreign_keys = ON;

-- ─── Members (stub) ──────────────────────────────────────────────────────────
--
-- Minimal table: only the primary key. This satisfies the foreign-key
-- reference from inbox_items without implementing the full Member entity.
--
-- DELIBERATE SHORTCUT: a single placeholder member with a known UUID is
-- inserted below. Production member management (creation, PINs, NFC auth,
-- two-tier governance) lands in a later task. The shortcut is documented
-- here and in the task description so it is not mistaken for the final design.

CREATE TABLE IF NOT EXISTS members (
    id   TEXT NOT NULL PRIMARY KEY   -- UUID v7, e.g. "018f1a2b-..."
) STRICT;

-- Insert the placeholder member. The UUID is fixed so that:
--   1. The prototype can capture inbox items without a member-management UI.
--   2. Tests can reference the same UUID without per-run coordination.
--   3. The migration is idempotent (INSERT OR IGNORE).
--
-- This UUID will remain valid in the database after the real Member entity
-- is introduced; the migration that introduces Members will either adopt it
-- as a sentinel or provide a data-migration path. Do not delete it without
-- a forward migration.
INSERT OR IGNORE INTO members (id)
VALUES ('00000000-0000-7000-8000-000000000001');

-- ─── Inbox items ─────────────────────────────────────────────────────────────
--
-- Corresponds to InboxItem in amity-core::inbox and brief §6.3.
--
-- Columns omitted from the prototype (arrived later):
--   • attachments[] — file storage adds a second concern; separate task.

CREATE TABLE IF NOT EXISTS inbox_items (
    id            TEXT     NOT NULL PRIMARY KEY,
    raw_text      TEXT     NOT NULL,
    captured_by   TEXT     NOT NULL REFERENCES members(id),
    captured_at   TEXT     NOT NULL,   -- ISO-8601, e.g. "2026-05-25T10:00:00Z"
    source        TEXT     NOT NULL,   -- snake_case InboxSource variant
    triage_state  TEXT     NOT NULL DEFAULT 'untriaged',
    triaged_to    TEXT                 -- NULL unless triage_state = 'typed'
) STRICT;

-- Index on captured_at DESC supports the list_recent_inbox_items query
-- without a full-table scan. DESC because we almost always want newest first.
CREATE INDEX IF NOT EXISTS idx_inbox_items_captured_at
    ON inbox_items (captured_at DESC);

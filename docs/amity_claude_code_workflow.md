# Amity — Claude Code Workflow

*How implementation work is delegated to Claude Code from the Claude Project, what context Claude Code needs to do good work, and how the human maintainer stays in the loop.*

---

## The handoff model

The Claude Project is the **design and planning** environment. The human maintainer works there, with full conversation context, to think through scope, design decisions, and architectural questions. The Project produces well-scoped task descriptions ready for implementation.

Claude Code is the **implementation** environment. It runs in the project repository, with full filesystem and tool access, and produces actual code, tests, and commits. It receives one well-scoped task at a time.

The maintainer is the bridge between the two. The Project produces a task; the maintainer initiates a Claude Code session with that task; Claude Code does the work; the maintainer reviews the result. The maintainer is not a courier — they are the editor.

## What a "well-scoped task" looks like

A task ready for Claude Code:

- **Has a clear single deliverable.** "Implement the `PantryItem` repository" not "build the pantry feature".
- **Names the files it expects to touch.** `crates/amity-storage/src/pantry.rs`, migrations in `crates/amity-storage/migrations/`. Not necessarily exhaustive, but a clear starting point.
- **References the relevant section of the brief** so Claude Code can verify decisions are honoured.
- **States the acceptance criteria.** "Tests exist for the three failure modes named in section 9.5 of the brief. `cargo clippy --pedantic` passes. Doc comments on all public items."
- **Calls out any non-obvious context** — known gotchas, related work in flight, decisions made in the conversation that aren't in the documents yet.

A task that's not yet ready for Claude Code:

- "Make pantry levels work." (Too vague.)
- "Refactor the storage layer." (Not a deliverable — what's the success criterion?)
- "Add some sync." (Architectural decision not yet made.)

When a task isn't ready, the right move is to keep it in the Project and develop it further, not to hand it to Claude Code and hope it figures things out.

## Context loading in Claude Code

When starting a Claude Code session, the first thing in the conversation is a context preamble:

```
This is the Amity project. The authoritative documents are:

- docs/amity_brief.md — the canonical design.
- docs/amity_philosophy.md — the project's values.
- docs/coding_guidelines.md — language-agnostic coding posture.
- docs/rust_guidelines.md — Rust-specific patterns and crate choices.

Before writing any code, read the relevant section of the brief
and confirm the design decisions are honoured. Comment density target
is 50%+ as specified in coding_guidelines.md.
```

This preamble can be kept as a file (`.claude-code-preamble.md`) at the project root and copied into the conversation by the maintainer.

Then the actual task description follows.

## Working agreements during a Claude Code session

**Read before writing.** Before producing code that touches a part of the system, Claude Code reads the relevant brief section and any existing code in that area. "Quick wins" that skip reading produce code that drifts from the design.

**Surface conflicts.** If the task description seems to conflict with the brief or philosophy, Claude Code stops and asks rather than picking a side. The maintainer arbitrates.

**Small commits.** Claude Code commits incrementally as work progresses, not in one giant change at the end. Each commit is a logical unit (a function with its tests; a migration with its rollback notes) and has a Conventional Commits-formatted message.

**Tests with code.** Tests are written alongside the code they test, not deferred to a "tests pass" pass. If a function is hard to test, that's a signal the function may be poorly designed; surface it rather than skipping the test.

**Comment density during writing, not after.** Comments are part of the code, not an afterthought. Code that lacks the required comment density is incomplete, not "ready for review".

**No silent dependency additions.** Adding a new crate to `Cargo.toml` is itself a decision; if the crate isn't in the rust-guidelines preferred list, Claude Code stops and explains why the new dependency is justified, then waits for maintainer confirmation.

## Review checkpoints

The maintainer reviews Claude Code's work at three points:

**Initial plan.** Before writing significant code, Claude Code outlines the approach (which files, which crates, which sequence). The maintainer either accepts or redirects. This catches misunderstandings before they become written code.

**Per-commit review.** As commits land, the maintainer can scan them quickly to verify direction. Major issues caught here are cheap; the same issues caught at the end are expensive.

**Final review.** The full task complete. Tests pass, lints clean, comments dense. The maintainer reads the diff, runs the code locally if possible, and either merges or sends back with feedback.

The middle checkpoint is the most often skipped and the most valuable. Even a 30-second skim of each commit catches drift early.

## Common task types and how to scope them

### "Implement entity X"

A typical entity implementation task includes:

- The data type definition in `amity-core`.
- A repository module in `amity-storage` with the basic CRUD functions.
- A migration creating the necessary tables.
- Unit tests for the type's invariants.
- Integration tests for the repository functions.
- Doc comments on every public item, linking to the relevant brief section.

Acceptance: `cargo test --workspace` passes, `cargo clippy --pedantic` passes, `cargo doc` builds clean, comment density audit passes.

### "Wire up endpoint Y"

A typical endpoint implementation task includes:

- The handler function in `amity-service`.
- The request/response types with serde implementations.
- Route registration in the service's router setup.
- Integration test exercising the endpoint end-to-end.
- Documentation in the API surface docs.

Acceptance: the integration test passes, the endpoint is reachable from a curl session against a local instance, the OpenAPI documentation generates without errors.

### "Fix bug Z"

A bug fix task includes:

- A regression test that reproduces the bug (and fails before the fix).
- The fix itself.
- A note in the relevant module's comments if the fix involves a subtle case that future readers would benefit from understanding.

Acceptance: the regression test fails before the fix, passes after; existing tests still pass.

### "Refactor module M"

Refactor tasks are higher risk and need more careful scoping:

- A clear statement of what changes and what does not.
- A test suite that exercises the public interface in enough detail to prove behaviour is preserved.
- Incremental commits, each independently passing tests, so the refactor can be bisected if something goes wrong.

Acceptance: behaviour is observably identical; tests still pass; the diff is reviewable in one sitting.

## When Claude Code is the wrong tool

Some tasks should stay in the Project rather than going to Claude Code:

- **Design conversations** where the right answer isn't yet known.
- **Architectural decisions** that warrant an ADR.
- **Cross-cutting changes** that touch many areas of the codebase — these need human judgment about scope before they're delegated.
- **Anything that touches the philosophy commitments.** Those decisions are not delegable; the maintainer makes them.

When in doubt, develop the task more in the Project before handing it off. Claude Code is most effective when the design is settled and the implementation is the remaining work.

## Tooling expected in the Claude Code session

The project repository ships with tooling that Claude Code uses:

- `cargo test --workspace` — run the test suite.
- `cargo clippy --workspace --all-targets -- -W clippy::pedantic` — lint check.
- `cargo fmt --all` — format code.
- `cargo doc --workspace --no-deps --all-features` — build documentation; fails on missing docstrings.
- `scripts/comment-density.sh <file>` — audit comment density on a specific file. (Custom script; lives in repo.)
- `scripts/check-philosophy.sh` — scan staged changes for common philosophy violations (calls to telemetry libraries, advertising code patterns, etc.). Lightweight grep-based check; supplementary to human review.

These should be run before claiming a task is complete.

## Working with the human

The maintainer has one set of priorities and Claude Code has another. The right working pattern:

- **Claude Code defers to the maintainer on direction.** When in doubt about scope, decisions, or what the maintainer wants — ask.
- **Claude Code defers to the documents on values.** When in doubt about whether something fits the project, the philosophy and brief are authoritative.
- **The maintainer defers to Claude Code on idiomatic implementation.** Rust patterns, async correctness, error handling — these are areas where Claude Code's competence is higher.
- **Both push back when they think the other is wrong.** Quiet agreement that produces bad code is worse than visible disagreement that gets resolved.

---

*This workflow document is itself a starting point. It will evolve as the project gains experience with the Claude Code pattern. Significant changes go through the maintainer and are recorded in the changelog.*

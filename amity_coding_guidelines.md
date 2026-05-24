# Amity — Coding Guidelines

*Language-agnostic posture for all code in the Amity project. Rust-specific guidance is in `rust_guidelines.md`; Claude Code workflow is in `claude_code_workflow.md`.*

---

## Posture

Amity is a system that people will run in their homes, on hardware they own, for years. The code should be readable by someone who comes to it cold, including the future maintainer six months from now and contributors who join along the way.

Three values drive these guidelines:

**Code is read more than written.** Optimise for the reader. Verbose names are usually better than short ones; explanatory comments are usually better than clever code. The cost of typing a longer name once is trivial compared to the cost of someone misunderstanding the code a year later.

**Be honest about uncertainty.** When a piece of code is doing something subtle, the comment explains *why*. When a function makes an assumption, the assumption is stated. When something is a workaround for a known limitation, the workaround says so and links to the issue.

**Respect the constraints in the philosophy.** No analytics, no telemetry, no third-party data flow except as explicitly architected. When in doubt, less network is better than more.

## Comment density

This is the single most concrete rule and the one most likely to be relaxed under pressure. Don't relax it.

**Target: at least 50% of source lines are comments or docstrings.** This is not a soft suggestion; it is a structural requirement of the codebase.

Specifically:

- **Every function** carries a docstring block before it. The block describes what the function does, what it expects, what it returns, what it does not handle, and any non-obvious context. Even small helper functions get this treatment.
- **Every non-trivial line** of code has an inline or following comment explaining *why*. "Why" not "what" — the code says what. The comment says why this approach was chosen, what alternatives were rejected, what edge cases it handles.
- **Every module** has a header comment explaining the module's purpose, its place in the larger system, and what other modules depend on it.
- **Every type** has a comment describing what it represents in the domain. For data model types, the comment links to the relevant section of the canonical brief.

Examples of comments that are doing real work:

```rust
// We use UUID v7 here (time-ordered) rather than v4 (random) because
// index locality matters at the scale of years of household data.
// See ADR-003.
let id = Uuid::now_v7();
```

```rust
// Saturating-subtract because a Presence window that ends in the past
// is valid (we keep historical Presence for audit/log purposes); a
// negative duration would be a bug.
let remaining = window.until.saturating_duration_since(now);
```

Examples of comments that are doing no work and should be deleted:

```rust
// Increment counter
counter += 1;
```

```rust
// Set name to the new name
self.name = new_name;
```

The rule is not "more comments are always better"; the rule is "comments earn their keep by explaining *why*". The 50% density target is a forcing function to make sure the *why* gets written down.

## Naming

**Variables**: full English words. `task_completion_timestamp`, not `tct`. `pantry_item_threshold`, not `pit`. Single-letter names are acceptable only for tight loops over collections (`for member in members`) or for established mathematical idioms.

**Functions**: verb-first, descriptive. `compute_chore_assignment_default()`, not `gca()`. The name should tell the reader what the function does without needing to read the body.

**Types**: noun, singular, matching the entity name in the canonical brief where applicable. `Task`, `Event`, `PantryItem`, `DietaryProfile`. The data model section of the brief is authoritative for entity names.

**Modules**: lowercase, snake_case, describing the domain. `inbox`, `surfacing`, `presence`, `pantry`.

**Avoid abbreviations** unless they are universal (HTTP, JSON, URL, UUID, ID, CRUD). Domain-specific abbreviations are not allowed even if "everyone in the project knows them" — future contributors don't.

## Errors and edge cases

**Errors are part of the contract.** Every function that can fail returns a typed error indicating why. Errors propagate honestly; nothing is swallowed silently.

**No `unwrap()` or equivalents in production code paths.** Use of `unwrap()` in production code requires a comment explaining why the invariant guarantees safety, and is reviewed skeptically. Test code may use `unwrap()` freely.

**Edge cases are named and handled.** Empty inputs, time-zone transitions, members with no presence record, recurring events crossing DST — these are not "exceptions to handle later". Each is acknowledged in code with explicit handling and a comment.

## Tests

**Every module has tests.** No exceptions for "trivial" modules; if it's worth writing, it's worth testing. Test files mirror source files: `src/pantry.rs` has tests in `src/pantry.rs` (within a `#[cfg(test)] mod tests` block) and integration tests in `tests/pantry_integration.rs` if cross-module behaviour is being verified.

**Tests are documentation.** A reader who wants to understand how a module is used should be able to read the tests and learn. Test names are descriptive sentences (`test_pantry_item_drops_to_grocery_list_when_level_below_threshold`), not numbered or terse.

**Tests follow the same comment density rule.** Test code is read too. Test code that nobody understands is test code that gets disabled the first time it fails.

## Architecture rules

**Local-first by default.** Any network operation requires explicit justification. Any third-party network operation requires triple justification: why it's needed, why the alternative (user-initiated, manual, deferred) is insufficient, and what privacy implications it carries. These justifications go in the commit message and the relevant ADR.

**Separation of concerns: storage, service, frontend.** The frontend talks to the service over a defined API; the service talks to storage through a repository abstraction. The data layer does not assume "the device is the whole thing" — the home-node trajectory in the brief requires this separation.

**Schema portability.** SQLite is the default backing store; Postgres is also supported (see brief section 5). Schema designs avoid features specific to one (Postgres JSONB operators, SQLite-specific pragmas) without abstraction. sqlx's migration machinery handles both.

**No global mutable state.** Configuration is loaded once and passed explicitly. The service is structured so that an integration test can spin it up with a clean state, a fixed clock, and a fresh database — repeatedly, in parallel.

## Architectural Decision Records (ADRs)

Significant decisions get an ADR. An ADR is a short markdown file (`docs/adrs/NNN-short-title.md`) describing:

- The decision being made.
- The context (what problem prompted it).
- The alternatives considered.
- The chosen approach.
- The consequences (good and bad).

ADRs are short — half a page is typical, two pages is the ceiling. They are dated and numbered. They are not edited after acceptance; superseded decisions get a new ADR that references the old one.

What needs an ADR:
- Choice of major dependency (web framework, ORM, async runtime).
- Significant deviation from an established pattern.
- Anything that future contributors will reasonably ask "why did we do it this way?".
- Anything that touches the philosophy commitments.

What does not need an ADR:
- Routine implementation choices within an established pattern.
- Minor refactors.
- Bug fixes that don't change architecture.

## Code review

**Every PR is reviewed by someone other than the author**, including when the maintainer is the author (in which case the review is by another contributor or, for solo phases, by Claude with explicit "review this change" prompts).

**Reviews check three things:**

1. Does the code do what the PR description says?
2. Is it consistent with the philosophy, the brief, and these guidelines?
3. Is the comment density adequate, and do the comments explain *why*?

A PR that fails on any of these gets feedback, not approval.

**Reviewers can be wrong.** If a reviewer requests a change that conflicts with the philosophy or the brief, the author should push back and link to the relevant section. Authority lives in the documents, not in role.

## Commits

**Commit messages follow the [Conventional Commits](https://www.conventionalcommits.org/) format.** This isn't pedantry; it lets us generate sensible changelogs and makes the history readable. Examples:

```
feat(inbox): add forwarding-email source for inbox items
fix(presence): correct timezone handling at DST transition
docs(philosophy): clarify the no-monitoring commitment
refactor(pantry): extract threshold computation into a helper
```

**Commits are signed off** using DCO (`git commit -s`). This certifies the contributor's right to submit the code under the project's license. See the licensing document.

**Commits are small.** A commit does one thing. If a change touches multiple concerns, it gets split. The pre-commit hooks check for unrelated changes within a single commit and warn.

## Tooling baseline

The repository ships with:

- A linter configuration enforced in CI.
- A formatter configuration applied by pre-commit hooks (no formatting debates in code review).
- A test runner that runs unit and integration tests on every PR.
- A documentation builder that fails CI if a public symbol is missing its docstring.

Tooling is set up once and left alone. The friction is in writing it; the value is in never having to think about it again.

---

*These guidelines are the operational expression of the philosophy. If a guideline ever conflicts with the philosophy, the philosophy wins and the guideline is wrong. Surface the conflict; don't paper over it.*

# Contributing to Amity

## Developer Certificate of Origin

All commits must be signed off with the Developer Certificate of Origin (DCO).
Add `-s` to your commit command:

```
git commit -s -m "feat(inbox): add forwarding-email source"
```

This certifies that you have the right to submit the code under the project's
AGPL-3.0 license. See [the DCO](https://developercertificate.org/) for the
full text.

## Commit format

Commits follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(inbox): add forwarding-email source for inbox items
fix(presence): correct timezone handling at DST transition
docs(philosophy): clarify the no-monitoring commitment
refactor(pantry): extract threshold computation into a helper
```

Commits are small. A commit does one thing. If a change touches multiple
concerns, split it.

## Running the test suite

```
cargo test --workspace
cargo clippy --workspace --all-targets -- -W clippy::pedantic
cargo fmt --check
cargo doc --workspace --no-deps --all-features
scripts/comment-density.sh <file>   # audit a specific Rust file
```

All of these must pass before a PR is mergeable. CI checks the same things.

## Comment density

The target is ≥ 50% comment lines per Rust source file. This is a structural
requirement, not a soft suggestion. See `docs/amity_coding_guidelines.md` for
the rationale and examples of comments that earn their keep.

## Philosophy

Before contributing a feature, read `docs/amity_philosophy.md`. The philosophy
is load-bearing: features that conflict with it are out of scope by definition.
The brief (`docs/amity_brief.md`) is the authoritative design document.

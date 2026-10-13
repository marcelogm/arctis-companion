# One Rust xtask enforces every engineering rule

Status: accepted (2026-10-11), revised 2026-10-12

The no-comments rule (every language, doc comments included), the ~400-line guideline (warning only), pure-core dependencies, durable doc paths and symbols, the extension XML copy, the ban on `pub unsafe fn`, and conventional commits without attribution (in commit messages, PR titles and PR bodies) are all checked by `cargo xtask`, used by git hooks and by CI. Clippy enforces cognitive complexity (threshold 8), function length (40), nesting (3) and argument count (4) as errors; ESLint does the same for the extension. The clippy lints that can only be satisfied by doc comments (`missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`, `undocumented_unsafe_blocks`, `doc_markdown`) are allowed, and `xtask` fails if those allows disappear.

One list of generated files (`Cargo.lock`, `cargo-sources.json`, `*.po`, `*.pot`, `po/LINGUAS`, `po/POTFILES*`, images) is exempt from both the comment check and the line limit. The ~400-line limit is a guideline set by the user: `xtask` reports it as a warning in CI and in the hook and never fails. A branch ruleset allows only rebase merging and requires the `rules` check, so GitHub never writes a squash or merge commit with its own `Co-authored-by:` trailers onto `main`; the `rules` job also runs on every push to `main`.

## Considered options

- Shell scripts and greps (the current `scripts/check-line-limit.sh` and `.githooks/`): cannot tell a comment from a string, and every script is another language under the same rules.
- Squash merging with a checked title: GitHub appends co-author trailers on its own.
- Pedantic clippy without allows: every public function returning `Result` would need a doc comment.

## Consequences

- `cargo xtask install-hooks` replaces the existing hooks and script.
- Rule fixtures are built in tempdirs by `xtask/tests`, never tracked.
- Research notes are dated records and exempt from the doc-path check; `docs/architecture/`, `CONTEXT.md` and ADR "Code:" lines are checked.
- Code: `xtask/src/check_rules/`, `xtask/src/check_commits.rs`, `xtask/src/check_doc_paths.rs`.

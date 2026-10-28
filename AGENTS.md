# Working on Yamark

Read [Development](docs/DEVELOPMENT.md) for setup and validation, [Architecture](docs/ARCHITECTURE.md) for ownership, and the relevant guide from the [documentation index](docs/README.md). Do not load every document for every task. Source and public acceptance tests settle disagreements with prose. `design-sketches/` contains historical experiments, not current requirements.

## Change discipline

Keep each behavior-changing commit focused on one observable change. Aim for fewer than 500 added/deleted implementation lines, but prefer a coherent change to an artificial split; mechanical moves, tests, fixtures, and docs are exempt. Keep mechanical refactors separate from behavior fixes.

Start behavior changes with a failing public regression. Establish an existing public baseline before internal refactors. Test public APIs, not private helpers. Preserve source spans, explicit byte-preservation boundaries, directive scope, and semantic verification through recursive formatting.

Reassess large files by responsibility rather than enforcing a line limit. Keep one Cargo package until a concrete crate boundary emerges. Reuse existing owners and keep new module visibility narrow. Keep straightforward, single-use code inline unless a helper names a meaningful responsibility or invariant.

Prefer named methods or enums when positional booleans or `None` obscure a call. Match owned state enums exhaustively. Add abstractions for implemented responsibilities, not planned features. Keep invariant comments and module documentation with extracted implementations.

Update the owning document when a contract changes. Explain decisions and invariants, not line-by-line mechanics. Document intentional limitations beside the owning code or contract. Keep user contracts separate from developer internals and consolidate unfinished work in [TODO](docs/TODO.md).

## Behavior tests

Every CLI-facing behavior change must include a CLI transcript case in `tests/cases/`, using the existing `.case` format documented in `tests/cases/README.md`. Record the exact invocation arguments, stdin, stdout, stderr, and exit status. Include relevant compatibility boundaries as well as the changed behavior.

For a bug fix, confirm that the new regression case fails on the affected revision and passes with the fix. Review expected output explicitly; do not regenerate unrelated cases. Run `cargo test --test cli_cases` after adding or changing cases. Parameterized integration tests may supplement these readable transcripts but do not replace them.

Editor-only behavior needs a regression through the public editor command or provider API in `editors/vscode/test/`. Use deferred promises to exercise edits and cancellation during formatting; a CLI transcript cannot observe editor document state.

Use temporary workspaces without changing `HOME` or the caller's tool environment. Prefer shared cross-platform transcripts; platform-specific cases must exercise a real platform difference. Keep exact errors, exit status, and meaningful ordering. Normalize incidental paths only after checking behavior. Use deterministic synchronization for subprocess tests, not timing assumptions. See the [test index](tests/README.md) for suite ownership.

## Validation and publishing

Run focused tests after editing, then `cargo fmt` and `git diff --check` before committing. Review formatter output and the resulting diff. Use `scripts/check.sh` as the final local gate; it runs the same checks as CI. See [Development](docs/DEVELOPMENT.md) for prerequisites and focused commands. Report commands actually run and unavailable coverage. Local success does not validate another platform; CI is the comprehensive merge gate.

Follow [RELEASE.md](RELEASE.md) before pushing a release tag. Never remove or replace published PyPI files. Respect the user's requested stopping point; do not infer merge or publication authorization.

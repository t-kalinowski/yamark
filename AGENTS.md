# Behavior tests

Every public-facing behavior change must include a CLI transcript case in
`tests/cases/`, using the existing `.case` format documented in
`tests/cases/README.md`. Record the exact invocation arguments, stdin, stdout,
stderr, and exit status. Include relevant compatibility boundaries as well as the
changed behavior.

For a bug fix, confirm that the new regression case fails on the affected revision
and passes with the fix. Review expected output explicitly; do not regenerate
unrelated cases. Run `cargo test --test cli_cases` after adding or changing cases.
Parameterized integration tests may supplement these readable transcripts but do
not replace them.

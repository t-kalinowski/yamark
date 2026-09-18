# CLI case files

Each `.case` file is executed through the compiled `yamark` binary. The format is:

```text
-- args
format --stdin-file-path input.md --wrap none
-- stdin
input text
-- stdout
expected stdout
-- stderr
expected stderr
-- status
0
```

The test harness intentionally avoids calling library internals. This keeps the public CLI contract readable and prevents implementation details from becoming part of the test API.

Generate expected output from the compiled CLI by setting `YAMARK_UPDATE_CASES` to the filename prefix of the affected cases, for example `YAMARK_UPDATE_CASES=markdown_inline_preservation cargo test --test cli_cases`. Other cases are checked without updating them. Review the resulting diff, then run `cargo test --test cli_cases` without the variable to check the snapshots. Keep byte-exact regression and idempotence assertions alongside snapshots; a generated snapshot alone does not establish correct formatting.

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

The test harness intentionally avoids calling library internals. This keeps the
public CLI contract readable and prevents implementation details from becoming
part of the test API.

`markdown_shortcode_nested_skip_file.case` verifies that `fmt: skip file`
preserves a nested Markdown fence/div body exactly, including trailing spaces in
shortcode arguments and ordinary prose. The `markdown_preserved_*.case`
transcripts cover explicit node/region preservation, nested fragments, the
linewise `fmt: on` boundary, and unchanged automatic fallback cleanup.
`tests/markdown_preservation.rs` supplements these transcripts with exact LF,
CRLF, CR, mixed-line-ending, EOF-without-newline, and second-pass checks.

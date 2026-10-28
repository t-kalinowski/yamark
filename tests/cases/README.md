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

The `markdown_nested_list_continuation_*` transcripts keep paragraphs after
nested lists inside their outer items. These layouts use the existing list
preservation behavior when they cannot be safely reflowed. The boundary case
also checks that sibling markers and top-level paragraphs still delimit list
items. `tests/nested_list_continuation.rs` checks line endings, wrap modes, and
second passes through the CLI.

The `markdown_list_shifted_sibling_lazy_continuation` and
`markdown_list_empty_item_pandoc_continuation` transcripts preserve layouts
whose partial rewrites change item membership. Empty-item indentation can parse
differently in GFM and Pandoc; preserving these layouts keeps both interpretations
intact.

The `markdown_list_following_raw_blocks` and `markdown_list_child_raw_blocks`
transcripts distinguish blocks outside a list from child blocks and lazy prose.
They also preserve block-like lines inside multiline code spans and links.

The `markdown_list_wide_marker_raw_boundary`, `markdown_list_child_headings`,
and `markdown_list_mixed_region_boundary` transcripts cover marker spacing,
child headings, and adjacent mixed list/raw regions. These ambiguous regions
remain unchanged, including marker padding; following outside prose can still be
formatted. `external-tests/corpus/test_scaling.py` checks that alternating lists
and fences do not repeatedly scan the remaining document.
`markdown_list_heading_literal_controls` keeps embedded formatting and inline
prose wrapping active around heading-like code content.

`markdown_shortcode_nested_skip_file.case` verifies that `fmt: skip file`
preserves a nested Markdown fence/div body exactly, including trailing spaces in
shortcode arguments and ordinary prose. The `markdown_preserved_*.case`
transcripts cover explicit node/region preservation, nested fragments, the
linewise `fmt: on` boundary, and unchanged automatic fallback cleanup.
`tests/markdown_preservation.rs` supplements these transcripts with exact LF,
CRLF, CR, mixed-line-ending, EOF-without-newline, and second-pass checks.

`markdown_literal_crlf.case`, `markdown_literal_cr.case`, and
`markdown_literal_mixed_endings.case` run `--diff` on byte-preserving fixtures in
`tests/fixtures/`. The case parser and diff display use LF. The fixtures retain
CRLF, bare CR, and mixed line endings, with Git attributes disabling checkout
conversion. The transcripts show literal whitespace retention and surrounding
formatting; `tests/multiline_literals.rs` checks the same fixtures' exact
formatted bytes and second passes. `markdown_literal_line_endings_check.case`
checks an already-formatted mixed-ending fixture, so a change to newline bytes
alone fails even when it would be invisible in the diff display.

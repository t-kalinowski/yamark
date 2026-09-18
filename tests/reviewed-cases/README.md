# Reviewed CLI cases

Recover the 13 authored transcript files from #4 and #6, preserving their complete
inputs and scenario order, plus two focused examples from the Python tests.
Active cases record the main revision below. **A baseline recording a limitation
does not endorse that behavior as permanently correct.**

| Source | Exact revision |
| --- | --- |
| Main / branch base | `0b1f072e710daa58ce4da412d8efc68f7d0fcd20` |
| [PR #4](https://github.com/t-kalinowski/yamark/pull/4) | `cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0` |
| [PR #6](https://github.com/t-kalinowski/yamark/pull/6) | `2e2caa4b14b8af0464a836988ec1967336762b78` |

The changed-file lists were checked against both PRs, including modified existing
cases. There are **15 new active cases: two matching and 13 differing**. None maps
to an identical existing main transcript. All 15 reviewed expectations reproduce
on their source heads; none is merely an unreproduced historical expectation.

## Source map

Each source link is pinned to the recorded head. Arguments and stdin are unchanged
in every recovered invocation. All runs have empty stderr and status 0. The
matching authored YAML case is copied unchanged. Differing authored transcripts
are copied byte-for-byte into `pr4/` or `pr6/`; their active counterparts record
main's stdout. Recovery filenames avoid overwriting existing cases, including
`markdown_template_code_boundaries.case` and `markdown_div_shortcode_math.case`.

| Source | Active case | Reviewed reference | Classification and difference |
| --- | --- | --- | --- |
| [#4: markdown_div_shortcode_math.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_div_shortcode_math.case) | [current](../cases/recovered_pr4_markdown_div_shortcode_math.case) | [reviewed](pr4/markdown_div_shortcode_math.case) | Unresolved: Matched shortcode bodies retain their authored spacing. |
| [#4: markdown_inline_code_template_sentence_wrap.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_inline_code_template_sentence_wrap.case) | [current](../cases/recovered_pr4_markdown_inline_code_template_sentence_wrap.case) | [reviewed](pr4/markdown_inline_code_template_sentence_wrap.case) | Policy: Complex bare expressions stay unchanged; eligible code and simple bare expressions reflow. |
| [#4: markdown_template_blocks_column_wrap.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_blocks_column_wrap.case) | [current](../cases/recovered_pr4_markdown_template_blocks_column_wrap.case) | [reviewed](pr4/markdown_template_blocks_column_wrap.case) | Unresolved: Markdown between matched shortcode tags stays unchanged. |
| [#4: markdown_template_code_boundaries.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_code_boundaries.case) | [current](../cases/recovered_pr4_markdown_template_code_boundaries.case) | [reviewed](pr4/markdown_template_code_boundaries.case) | Policy: The quoted shortcode/code example retains its paragraph layout. |
| [#4: markdown_template_column_wrap.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_column_wrap.case) | [current](../cases/recovered_pr4_markdown_template_column_wrap.case) | [reviewed](pr4/markdown_template_column_wrap.case) | Policy: Complex bare expressions and conditional tags stay unwrapped. |
| [#4: markdown_template_multiline_wrap.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_multiline_wrap.case) | [current](../cases/recovered_pr4_markdown_template_multiline_wrap.case) | [reviewed](pr4/markdown_template_multiline_wrap.case) | Policy: Prose surrounding multiline template code stays unwrapped. |
| [#4: markdown_template_prose_boundaries.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_prose_boundaries.case) | [current](../cases/recovered_pr4_markdown_template_prose_boundaries.case) | [reviewed](pr4/markdown_template_prose_boundaries.case) | Policy: Containers and nested emphasis/link contexts retain template preservation. |
| [#4: yaml_markdown_template_block_boundaries.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/yaml_markdown_template_block_boundaries.case) | [current](../cases/yaml_markdown_template_block_boundaries.case) | Same as active | Matches: Quoted YAML block boundaries are unchanged. |
| [#6: markdown_inline_preservation_canonical.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_canonical.case) | [current](../cases/recovered_pr6_markdown_inline_preservation_canonical.case) | [reviewed](pr6/markdown_inline_preservation_canonical.case) | Unresolved: Emphasis can change inside literals; literal line-end whitespace is trimmed. HTML link padding also follows #8. |
| [#6: markdown_inline_preservation_column.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_column.case) | [current](../cases/recovered_pr6_markdown_inline_preservation_column.case) | [reviewed](pr6/markdown_inline_preservation_column.case) | Unresolved: Math emphasis changes; the wide-list literal retains its layout and loses trailing spaces. |
| [#6: markdown_inline_preservation_multiline.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_multiline.case) | [current](../cases/recovered_pr6_markdown_inline_preservation_multiline.case) | [reviewed](pr6/markdown_inline_preservation_multiline.case) | Unresolved: Literal whitespace and container continuations differ. |
| [#6: markdown_inline_preservation_paragraph.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_paragraph.case) | [current](../cases/recovered_pr6_markdown_inline_preservation_paragraph.case) | [reviewed](pr6/markdown_inline_preservation_paragraph.case) | Unresolved: Literal whitespace is trimmed; an eligible code-template paragraph also reflows under #9. |
| [#6: markdown_inline_preservation_sentence.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_sentence.case) | [current](../cases/recovered_pr6_markdown_inline_preservation_sentence.case) | [reviewed](pr6/markdown_inline_preservation_sentence.case) | Unresolved: Wide-list literal whitespace and vertical-tab link normalization differ. |
| [#4: test_unsupported_inline_syntax_preserves_paragraph](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/external-tests/cli/test_template_spans.py#L225) | [current](../cases/recovered_pr4_mismatched_backticks.case) | Same as active | Matches: The two-opening/three-closing-backtick example remains unchanged. |
| [#4: test_shortcode_tags_leave_intervening_markdown_to_format](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/external-tests/cli/test_template_spans.py#L82) | [current](../cases/recovered_pr4_shortcode_quoted_close.case) | [reviewed](pr4/shortcode_quoted_close.case) | Unresolved: A quoted `%}}` closes the tag early; argument whitespace and surrounding layout change. |

The two Python selections retain their original `--verify` invocations: the third
`template` value in `test_unsupported_inline_syntax_preserves_paragraph`, and the
sixth opening/closing pair in `test_shortcode_tags_leave_intervening_markdown_to_format`
with LF. They add a malformed-backtick boundary and a multiline tag with a quoted
closing delimiter without importing the parameter cross-products.

**Matches** means the reviewed expectation is recovered unchanged. **Policy**
means the difference follows the narrower paragraph eligibility merged in
[#9](https://github.com/t-kalinowski/yamark/pull/9) and
[#10](https://github.com/t-kalinowski/yamark/pull/10). **Unresolved** means remaining
desired behavior or a suspected defect. Combined cases stay unresolved when they
also contain intentional differences, such as
[#8's raw-angle link policy](https://github.com/t-kalinowski/yamark/pull/8).

The reference directory is historical evidence, **not a second passing suite or
an alternative oracle**. Each current/reference pair shows the exact stdout
difference without a duplicated diff report. For example:

```sh
git diff --no-index tests/reviewed-cases/pr4/markdown_div_shortcode_math.case tests/cases/recovered_pr4_markdown_div_shortcode_math.case
```

That pair retains the same input, including `Format   this body as Markdown.`;
only stdout differs. Tabs, form feeds, vertical tabs, and literal trailing spaces
in these files are intentional bytes. Do not trim or regenerate the references.

## Additional source material

This is a focused transcript recovery, not an exhaustive import of the auxiliary
test matrices. The following source tests remain available at their recorded
heads. Unselected parameter variants and property assertions are not claimed as
new active coverage.

| Source | Purpose and boundary of this recovery |
| --- | --- |
| [#4: external-tests/cli/test_template_context.py](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/external-tests/cli/test_template_context.py) | HTML/template, attribute, container, decoded-YAML, and preservation variants were inspected; the authored transcripts retain the combined examples. Additional variants are not expanded. |
| [#4: external-tests/cli/test_template_spans.py](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/external-tests/cli/test_template_spans.py) | Two examples are selected above. Other wrap/canonical combinations, CRLF variants, temporary delimiter configuration, semantic checks, and second-pass assertions are not imported. |
| [#4: tests/spec_cli.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/spec_cli.rs) | The modified braced-heading and shortcode-body tests remain here; their exact individual invocations are not added separately. |
| [#6: tests/inline_preservation.rs](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/inline_preservation.rs) | The five authored transcripts retain its combined literal, whitespace, canonicalization, and container examples. Additional matrices and idempotence assertions are not imported. |
| [#4: external-tests/corpus/test_scaling.py](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/external-tests/corpus/test_scaling.py) | Five timing functions (11 variants) remain here: flow-heavy YAML, unmatched backticks, nested brackets, incomplete HTML, and unmatched braces in HTML. No performance measurements are imported. |

The unchanged `.case` harness normalizes line endings and restores a final LF;
it cannot faithfully encode raw CRLF/CR, mixed endings, or unterminated stdin.
It also has no temporary configuration/file setup, semantic-equivalence check,
second-pass assertion, or timing support. Those purposes remain explicit in the
pinned sources; they are not silently converted to LF or smaller inputs here.
The timing tests require post-format file assertions and `wait4` CPU ratios.

Other changed tests concern runner/check-script assertions or removal of the old
YAML timing helper; they add no other formatter transcripts. Production, harness,
dependency, CI, and product-documentation changes from both old PRs are excluded.

## Validation

The retained cases were replayed against main and their recorded source heads,
comparing exact stdout, stderr, and status. `cargo test --test cli_cases` passes.
Production files, the harness, and every pre-existing transcript remain unchanged.

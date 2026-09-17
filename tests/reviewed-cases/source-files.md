# Changed source files

These are the complete merge-base changed-file lists reported by Git and checked against the refreshed GitHub PR file lists. `A` means added; `M` means modified.

## PR #4 at `cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0`

| Change | Source file | Disposition |
| --- | --- | --- |
| M | [RELEASE_NOTES.md](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/RELEASE_NOTES.md) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [docs/markdown-support-workpad.md](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/docs/markdown-support-workpad.md) | Production, release, CI/check-script, or product documentation changes excluded. |
| A | [external-tests/cli/test_template_context.py](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/external-tests/cli/test_template_context.py) | All public CLI invocation matrices recovered; frozen source retained. |
| A | [external-tests/cli/test_template_spans.py](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/external-tests/cli/test_template_spans.py) | All public CLI invocation matrices recovered; frozen source retained. |
| A | [external-tests/corpus/test_scaling.py](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/external-tests/corpus/test_scaling.py) | Frozen timing/file assertions; exact limitations and purpose listed in the index. |
| M | [scripts/check.sh](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/scripts/check.sh) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [src/core/directives.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/src/core/directives.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [src/core/emit.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/src/core/emit.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [src/core/markdown.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/src/core/markdown.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [src/core/wrap.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/src/core/wrap.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [src/core/yaml.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/src/core/yaml.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [tests/benchmark_tools.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/benchmark_tools.rs) | Only deletes the old YAML wall-clock scaling test/helper in favor of external child-CPU checks. Main test remains unchanged. |
| M | [tests/cases/markdown_div_shortcode_math.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_div_shortcode_math.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_inline_code_template_sentence_wrap.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_inline_code_template_sentence_wrap.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_template_blocks_column_wrap.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_blocks_column_wrap.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_template_code_boundaries.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_code_boundaries.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_template_column_wrap.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_column_wrap.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_template_multiline_wrap.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_multiline_wrap.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_template_prose_boundaries.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/markdown_template_prose_boundaries.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/yaml_markdown_template_block_boundaries.case](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/cases/yaml_markdown_template_block_boundaries.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| M | [tests/python_cli.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/python_cli.rs) | Runner parallelism or shared-check command assertions only; no formatter scenarios added. Excluded. |
| M | [tests/release_scripts.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/release_scripts.rs) | Runner parallelism or shared-check command assertions only; no formatter scenarios added. Excluded. |
| M | [tests/spec_cli.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/spec_cli.rs) | Both changed heading and shortcode-body public CLI tests recovered. |
| M | [tests/website.rs](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/tests/website.rs) | Runner parallelism or shared-check command assertions only; no formatter scenarios added. Excluded. |
| M | [website/reference-config.html.md](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/website/reference-config.html.md) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [website/reference-config.qmd](https://github.com/t-kalinowski/yamark/blob/cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0/website/reference-config.qmd) | Production, release, CI/check-script, or product documentation changes excluded. |

## PR #6 at `2e2caa4b14b8af0464a836988ec1967336762b78`

| Change | Source file | Disposition |
| --- | --- | --- |
| M | [RELEASE_NOTES.md](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/RELEASE_NOTES.md) | Production, release, CI/check-script, or product documentation changes excluded. |
| A | [docs/inline-formatting.md](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/docs/inline-formatting.md) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [src/core/emit.rs](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/src/core/emit.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [src/core/markdown.rs](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/src/core/markdown.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [src/core/wrap.rs](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/src/core/wrap.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| A | [src/core/wrap/inline.rs](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/src/core/wrap/inline.rs) | Production, release, CI/check-script, or product documentation changes excluded. |
| M | [tests/cases/README.md](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/README.md) | Documentation of snapshot-regeneration switch. That switch is not imported. |
| A | [tests/cases/markdown_inline_preservation_canonical.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_canonical.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_inline_preservation_column.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_column.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_inline_preservation_multiline.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_multiline.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_inline_preservation_paragraph.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_paragraph.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| A | [tests/cases/markdown_inline_preservation_sentence.case](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cases/markdown_inline_preservation_sentence.case) | Recovered complete invocation and scenario order; active/reference map in the index. |
| M | [tests/cli_cases.rs](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/cli_cases.rs) | Snapshot-regeneration switch only. Harness change excluded. |
| A | [tests/inline_preservation.rs](https://github.com/t-kalinowski/yamark/blob/2e2caa4b14b8af0464a836988ec1967336762b78/tests/inline_preservation.rs) | All public CLI invocation matrices recovered; frozen source retained. |

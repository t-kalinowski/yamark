# Reviewed CLI case recovery

This corpus recovers behavior reviewed in [#4](https://github.com/t-kalinowski/yamark/pull/4) and [#6](https://github.com/t-kalinowski/yamark/pull/6). Active transcripts describe the recorded main revision. Historical references preserve the reviewed expectations. **Recording a current limitation does not endorse it as permanently correct.**

| Revision                                 | Exact commit                               |
| ---------------------------------------- | ------------------------------------------ |
| Main used for comparison and branch base | `0b1f072e710daa58ce4da412d8efc68f7d0fcd20` |
| #4, `fix/template-code-spans`            | `cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0` |
| #6, `fix/shared-inline-pipeline`         | `2e2caa4b14b8af0464a836988ec1967336762b78` |

The PR heads and changed-file lists were refreshed on 2026-09-17. Each diff uses its merge base with main, `bfa7c7b6446e3ab870bd54daf8d6962f5a944343`. Neither old branch was merged, rebased, or cherry-picked. The two old executables were built from separate archived trees and separate build directories. Comparisons used the public CLI with exact UTF-8 input/output bytes, including the original argument order, `--verify` where present, empty stderr, status 0, and temporary configuration where required.

## Corpus and counts

| Material                                                 |   #4 |    #6 | Total |
| -------------------------------------------------------- | ---: | ----: | ----: |
| Added/modified authored `.case` files                    |    8 |     5 |    13 |
| Other public CLI test functions expanded                 |   43 |    18 |    61 |
| Timing functions retained as historical source only      |    5 |     0 |     5 |
| Source invocations, including reviewed-output replay     |  728 | 4,141 | 4,869 |
| Distinct invocations after exact deduplication           |  723 | 3,810 | 4,533 |
| New active transcripts                                   |  237 |   424 |   661 |
| Equivalent pre-existing main transcripts                 |    0 |     0 |     0 |
| Active transcripts with a differing reviewed expectation |  102 |   180 |   282 |
| Distinct invocations matching reviewed expectations      |  556 | 2,108 | 2,664 |
| Distinct invocations differing under a merged policy     |  116 |    52 |   168 |
| Distinct invocations with unresolved differences         |   51 | 1,650 | 1,701 |
| Historical exact expectations not reproduced             |    0 |     0 |     0 |

All 13 authored transcripts retain their complete scenario order. Twelve differ from main; their original files are copied byte-for-byte into the reference directories. The matching YAML transcript is copied byte-for-byte into the active corpus. Other matching generated transcripts retain the source arguments, input, and expectations. Filenames prefixed with `recovered_pr4_` or `recovered_pr6_` avoid collisions, including the existing `markdown_template_code_boundaries.case` and `markdown_div_shortcode_math.case`. No existing transcript is overwritten.

There are 379 matching and 282 differing new transcripts. The 282 references are historical evidence, **not a second passing suite or an alternative oracle**. Do not run or regenerate them as current expectations. Their source and active counterpart are linked in the maps below.

The existing Python CLI helper executes all 4,533 distinct invocations in 1,340 parameter groups. Each group shares input and exact expectations while retaining each distinct command. The transcripts select readable first-pass syntax/behavior examples; remaining options and reviewed-output replay stay executable in these matrices. This avoids repeating the same example for every wrap/canonical combination. It is matrix factoring, not a claim that different options are equivalent. All 4,869 source calls retain provenance in the maps, including the 336 exactly repeated invocations. Existing integration coverage is not counted as transcript equivalence.

- [#4 source-to-active map](pr4/mapping.md) and [exact differences](pr4/differences.md).
- [#6 source-to-active map](pr6/mapping.md) and [exact differences](pr6/differences.md).
- Active option and byte matrices: [#4](../../external-tests/cli/test_reviewed_pr4_options.py), [#6](../../external-tests/cli/test_reviewed_pr6_options.py).

Each mapping row names its source file/test, source variant and invocation, active transcript where applicable, exact matrix commands, reviewed reference, and classification. Every source link is pinned to the recorded SHA. The difference pages show readable stdout diffs and escaped exact stdin/stdout/stderr/status, including otherwise invisible whitespace. Matrix-only differences are preserved there without pretending that raw CR bytes fit the `.case` format.

## Reading the classifications

`matches` means recovered and matches the reviewed expectation. `policy` means differs because of an intentional merged policy. `unresolved` means remaining desired behavior or a suspected defect; it is not an approval of the current result. `already covered identically` would require equal arguments, bytes, configuration, stdout, stderr, and status in an existing transcript; none qualified. No historical exact expectation failed to reproduce on its source head.

The source heads reproduced all reviewed exact stdout, stderr, and status expectations. One #6 function, `inline_pipeline_wraps_around_overwide_literals`, authored containment, real-link normalization, and idempotence assertions rather than full stdout. Its 30 examples and 30 second calls passed those original assertions on the old executable. Their full transcript output is **observed source-head output**, not an authored exact-output expectation; main produced the same bytes. The mapping labels these rows.

- **Merged #8 policy:** unescaped raw angle syntax makes link normalization retain padding throughout that input unit. This explains the HTML real-link differences; it does not justify damage to literal spaces or delimiters. See [the merged change](https://github.com/t-kalinowski/yamark/pull/8) and [`pr6-167`](pr6/differences.md#pr6-167).
- **Merged #9/#10 policy:** only eligible ordinary paragraphs reflow around complete single-line top-level code or the fixed simple bare-template subset. Containers, host languages, nested contexts, quoted/complex/attached expressions, multiline code, and ambiguous delimiters keep their guards. A layout that would isolate a bare expression also preserves the paragraph. This intentionally narrower policy explains many #4 differences and supersedes #6's old simple-template preservation expectations. See [#9](https://github.com/t-kalinowski/yamark/pull/9), [#10](https://github.com/t-kalinowski/yamark/pull/10), and [the old policy example](pr6/mapping.md#pr6-154). It does not settle whether the broader #4 behavior should be added later.
- **Unresolved literal behavior:** main can trim or rewrite spaces at a literal's physical line end, collapse multiline code in containers, or alter emphasis delimiters inside code/math. [Canonical emphasis](pr6/differences.md#pr6-210) and [literal line endings](pr6/differences.md#pr6-171) show exact examples. The current `wrap.rs` hard-break/line cleanup and independent canonical-emphasis scanning explain these differences; this recovery does not fix them. Vertical-tab handling and continuation indentation differences also remain unresolved.
- **Unresolved shortcode-body policy:** main preserves bodies that #4 treats as intervening Markdown. [The shortcode examples](pr4/differences.md#pr4-297) preserve matching, unrelated, multiline, quoted-delimiter, and backtick-argument tags. The authored `markdown_div_shortcode_math.case` uses `Format   this body as Markdown.` That wording is retained even though main does not format it. The maintainer still needs to decide the body policy.

Some combined transcripts contain both intentional and unresolved differences. These are classified `unresolved`; their full diffs remain visible. In particular, #6's combined canonical/paragraph cases include raw-angle policy or new template reflow alongside literal damage.

## Bytes, configuration, and properties outside `.case`

The unchanged harness uses Rust `str::lines()`, restores LF after each parsed line, splits arguments on whitespace, and has no per-case file setup. It cannot faithfully express the following source invocations. They remain active through the existing byte-preserving Python helper:

| Boundary                                              | Source calls | Representation                                                                                                                                                                |
| ----------------------------------------------------- | -----------: | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| #4 CRLF                                               |           54 | Exact `\r\n` input and expected output in the #4 matrix; no LF substitution                                                                                                   |
| #6 CRLF, CR, and mixed endings in recursive footnotes |          720 | Exact escaped bytes in the #6 matrix, including independently authored structural LF                                                                                          |
| #6 missing final newline (`first\u000c`)              |            5 | Exact unterminated stdin in the #6 matrix                                                                                                                                     |
| #4 temporary `yamark.toml`                            |            2 | Original `[template]` / `add_delimiters = [{ open = "<<", close = ">>" }]` setup in [`pr4-405`/`406`](pr4/mapping.md#pr4-405); the directive variant is a separate transcript |

Tabs, form feeds, vertical tabs, non-ASCII spaces, and trailing literal spaces that the harness can represent remain literal bytes in `.case` files. Do not trim them. The escaped difference pages provide an additional way to inspect them.

The 144 #4 calls originally using `format_stdin_and_check` retain that helper's semantic comparison as well as exact output checks. Other invocations retain the original exact-output contract. The original generators are frozen without edits as [template context](pr4/source/test_template_context.py.txt), [template spans](pr4/source/test_template_spans.py.txt), and [inline preservation](pr6/source/inline_preservation.rs.txt).

Reviewed-output replay retains the original input to the second invocation. When main differs, this is not necessarily main's own first output. Thus passing baseline tests do not assert that every current output is idempotent, semantically correct, or preserves literal bytes. They make each historical invocation and its actual current result inspectable.

The only source tests not recovered as executable assertions are the five timing functions (11 parameter variants) in [the frozen scaling source](pr4/source/test_scaling.py.txt):

| Source test                                                    | Original input sizes and assertion                                                        | Reason                                                                         |
| -------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `test_flow_heavy_yaml_formatting_scales_near_linearly`         | 400/1,600 generated YAML items; exact count of formatted ports; child CPU ratio at most 6 | Temporary files, post-format file contents, `wait4` child CPU accounting       |
| `test_unmatched_backticks_scale_with_input_size`               | 400/800 increasing backtick runs; file unchanged; CPU ratio at most 6                     | File mutation and CPU timing cannot be represented by stdin/stdout transcripts |
| `test_nested_brackets_scale_with_input_size`                   | 2,000/8,000 pairs, three labels, two opening forms; file unchanged; CPU ratio at most 6   | Six parameter variants require file and CPU assertions                         |
| `test_incomplete_html_before_templates_scales_with_input_size` | 20,000/80,000 comment openers; file unchanged; CPU ratio at most 6                        | File and CPU assertions                                                        |
| `test_unmatched_braces_in_inline_html_scale_with_input_size`   | 2,000/8,000 `{ ` or `{{ ` openers; expected `Before` line split; CPU ratio at most 6      | Two parameter variants require file and CPU assertions                         |

Their exact generators, `yamark format <temporary-path>` invocation, expected file content predicates, exit assertion, logging, and platform skip condition remain in the frozen source. They were not run as performance measurements or reduced to smaller inputs. The pre-existing main wall-clock YAML benchmark remains unchanged. This PR adds no timing tests, skips, dependencies, harness changes, or production fixes.

## Changed-file completeness

[The full changed-file inventory](source-files.md) records added **and modified** files from both PRs, including non-behavior files intentionally excluded from this tests-only recovery. The table below maps every recovered source test family; detailed parameter/command mappings are in the linked rows.

| PR  | Source file / test                                                                                               | Detailed mapping                  |
| --- | ---------------------------------------------------------------------------------------------------------------- | --------------------------------- |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_prose_boundaries`                                  | [pr4-001](pr4/mapping.md#pr4-001) |
| #4  | `external-tests/cli/test_template_context.py`: `test_templates_crossing_protected_fragments_are_preserved`       | [pr4-003](pr4/mapping.md#pr4-003) |
| #4  | `external-tests/cli/test_template_context.py`: `test_quoted_markdown_yaml_does_not_join_code_across_blocks`      | [pr4-009](pr4/mapping.md#pr4-009) |
| #4  | `external-tests/cli/test_template_context.py`: `test_html_template_blocks_are_preserved`                         | [pr4-012](pr4/mapping.md#pr4-012) |
| #4  | `external-tests/cli/test_template_context.py`: `test_html_template_headings_are_preserved`                       | [pr4-054](pr4/mapping.md#pr4-054) |
| #4  | `external-tests/cli/test_template_context.py`: `test_inline_shortcodes_keep_quoted_delimiters`                   | [pr4-058](pr4/mapping.md#pr4-058) |
| #4  | `external-tests/cli/test_template_context.py`: `test_code_spans_do_not_cross_deeply_indented_list_children`      | [pr4-082](pr4/mapping.md#pr4-082) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_blocks_with_apparent_hard_breaks_are_preserved`    | [pr4-090](pr4/mapping.md#pr4-090) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_pairs_crossing_code_spans_preserve_the_block`      | [pr4-120](pr4/mapping.md#pr4-120) |
| #4  | `external-tests/cli/test_template_context.py`: `test_braced_templates_crossing_source_lines_are_preserved`       | [pr4-134](pr4/mapping.md#pr4-134) |
| #4  | `external-tests/cli/test_template_context.py`: `test_braced_fig_alt_values_are_not_split`                        | [pr4-137](pr4/mapping.md#pr4-137) |
| #4  | `external-tests/cli/test_template_context.py`: `test_attached_templates_stay_in_one_token`                       | [pr4-141](pr4/mapping.md#pr4-141) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_detection_respects_link_label_boundaries`          | [pr4-157](pr4/mapping.md#pr4-157) |
| #4  | `external-tests/cli/test_template_context.py`: `test_generic_brace_groups_keep_literal_quotes`                   | [pr4-163](pr4/mapping.md#pr4-163) |
| #4  | `external-tests/cli/test_template_context.py`: `test_balanced_templates_crossing_child_blocks_are_preserved`     | [pr4-179](pr4/mapping.md#pr4-179) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_tag_names_do_not_change_markdown_formatting`       | [pr4-185](pr4/mapping.md#pr4-185) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_tags_inside_inline_code_allow_wrapping`            | [pr4-191](pr4/mapping.md#pr4-191) |
| #4  | `external-tests/cli/test_template_context.py`: `test_templates_in_raw_inline_html_preserve_the_block`            | [pr4-197](pr4/mapping.md#pr4-197) |
| #4  | `external-tests/cli/test_template_context.py`: `test_raw_html_spelling_inside_inline_code_still_allows_wrapping` | [pr4-209](pr4/mapping.md#pr4-209) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_in_quoted_html_attribute_is_not_code`              | [pr4-211](pr4/mapping.md#pr4-211) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_in_escaped_link_destination_is_not_code`           | [pr4-215](pr4/mapping.md#pr4-215) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_in_image_attributes_is_not_code`                   | [pr4-217](pr4/mapping.md#pr4-217) |
| #4  | `external-tests/cli/test_template_context.py`: `test_braced_templates_survive_image_attribute_normalization`     | [pr4-220](pr4/mapping.md#pr4-220) |
| #4  | `external-tests/cli/test_template_context.py`: `test_template_in_heading_attributes_is_not_code`                 | [pr4-224](pr4/mapping.md#pr4-224) |
| #4  | `external-tests/cli/test_template_context.py`: `test_quoted_braces_stay_inside_the_template`                     | [pr4-228](pr4/mapping.md#pr4-228) |
| #4  | `external-tests/cli/test_template_context.py`: `test_quoted_markdown_yaml_uses_decoded_backtick_escapes`         | [pr4-244](pr4/mapping.md#pr4-244) |
| #4  | `external-tests/cli/test_template_context.py`: `test_code_spans_do_not_cross_container_blocks`                   | [pr4-247](pr4/mapping.md#pr4-247) |
| #4  | `external-tests/cli/test_template_context.py`: `test_multiline_template_code_keeps_its_container_paragraph`      | [pr4-251](pr4/mapping.md#pr4-251) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_containers_preserve_multiline_code`                           | [pr4-253](pr4/mapping.md#pr4-253) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_multiline_template_code_wraps_by_its_first_and_last_lines`    | [pr4-285](pr4/mapping.md#pr4-285) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_shortcode_tags_leave_intervening_markdown_to_format`          | [pr4-297](pr4/mapping.md#pr4-297) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_wrap_around_braced_template_text`                             | [pr4-325](pr4/mapping.md#pr4-325) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_wrap_around_template_text_in_inline_code`                     | [pr4-333](pr4/mapping.md#pr4-333) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_template_delimiters_inside_complete_code_spans`               | [pr4-345](pr4/mapping.md#pr4-345) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_wrap_around_protected_template_tokens`                        | [pr4-373](pr4/mapping.md#pr4-373) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_unsupported_inline_syntax_preserves_paragraph`                | [pr4-399](pr4/mapping.md#pr4-399) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_custom_template_delimiters_inside_code`                       | [pr4-405](pr4/mapping.md#pr4-405) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_inline_code_templates_in_other_markdown_blocks`               | [pr4-409](pr4/mapping.md#pr4-409) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_nested_inline_code_templates_allow_wrapping`                  | [pr4-418](pr4/mapping.md#pr4-418) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_inline_code_templates_in_marked_markdown`                     | [pr4-422](pr4/mapping.md#pr4-422) |
| #4  | `external-tests/cli/test_template_spans.py`: `test_yaml_template_text_with_backticks_is_preserved`               | [pr4-446](pr4/mapping.md#pr4-446) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_preserves_literals_and_normalizes_real_links`                   | [pr6-001](pr6/mapping.md#pr6-001) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_preserves_multiline_literal_bytes_in_containers`                | [pr6-089](pr6/mapping.md#pr6-089) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_keeps_main_template_policy`                                     | [pr6-153](pr6/mapping.md#pr6-153) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_keeps_existing_markup_and_heading_normalization`                | [pr6-165](pr6/mapping.md#pr6-165) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_preserves_literal_line_endings`                                 | [pr6-171](pr6/mapping.md#pr6-171) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_wraps_around_overwide_literals`                                 | [pr6-180](pr6/mapping.md#pr6-180) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_canonical_emphasis_skips_literal_delimiters`                    | [pr6-210](pr6/mapping.md#pr6-210) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_preserves_definition_and_footnote_literals`                     | [pr6-214](pr6/mapping.md#pr6-214) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_preserves_multiline_literals_nested_in_markup`                  | [pr6-218](pr6/mapping.md#pr6-218) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_normalizes_editable_form_feeds`                                 | [pr6-304](pr6/mapping.md#pr6-304) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_canonical_emphasis_respects_escaped_openers`                    | [pr6-334](pr6/mapping.md#pr6-334) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_normalizes_gaps_after_backslashes`                              | [pr6-358](pr6/mapping.md#pr6-358) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_keeps_unsupported_whitespace_out_of_reflow`                     | [pr6-586](pr6/mapping.md#pr6-586) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_normalizes_list_separators_without_trimming_literals`           | [pr6-601](pr6/mapping.md#pr6-601) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_preserves_literals_in_recursive_footnotes`                      | [pr6-632](pr6/mapping.md#pr6-632) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_normalizes_nested_output_once`                                  | [pr6-674](pr6/mapping.md#pr6-674) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_accepts_definition_continuation_indentation`                    | [pr6-698](pr6/mapping.md#pr6-698) |
| #6  | `tests/inline_preservation.rs`: `inline_pipeline_accepts_list_literal_continuation_indentation`                  | [pr6-746](pr6/mapping.md#pr6-746) |
| #4  | `tests/cases/markdown_div_shortcode_math.case`: `markdown_div_shortcode_math`                                    | [pr4-448](pr4/mapping.md#pr4-448) |
| #4  | `tests/cases/markdown_inline_code_template_sentence_wrap.case`: `markdown_inline_code_template_sentence_wrap`    | [pr4-449](pr4/mapping.md#pr4-449) |
| #4  | `tests/cases/markdown_template_blocks_column_wrap.case`: `markdown_template_blocks_column_wrap`                  | [pr4-450](pr4/mapping.md#pr4-450) |
| #4  | `tests/cases/markdown_template_code_boundaries.case`: `markdown_template_code_boundaries`                        | [pr4-451](pr4/mapping.md#pr4-451) |
| #4  | `tests/cases/markdown_template_column_wrap.case`: `markdown_template_column_wrap`                                | [pr4-452](pr4/mapping.md#pr4-452) |
| #4  | `tests/cases/markdown_template_multiline_wrap.case`: `markdown_template_multiline_wrap`                          | [pr4-453](pr4/mapping.md#pr4-453) |
| #4  | `tests/cases/markdown_template_prose_boundaries.case`: `markdown_template_prose_boundaries`                      | [pr4-454](pr4/mapping.md#pr4-454) |
| #4  | `tests/cases/yaml_markdown_template_block_boundaries.case`: `yaml_markdown_template_block_boundaries`            | [pr4-455](pr4/mapping.md#pr4-455) |
| #6  | `tests/cases/markdown_inline_preservation_canonical.case`: `markdown_inline_preservation_canonical`              | [pr6-879](pr6/mapping.md#pr6-879) |
| #6  | `tests/cases/markdown_inline_preservation_column.case`: `markdown_inline_preservation_column`                    | [pr6-880](pr6/mapping.md#pr6-880) |
| #6  | `tests/cases/markdown_inline_preservation_multiline.case`: `markdown_inline_preservation_multiline`              | [pr6-881](pr6/mapping.md#pr6-881) |
| #6  | `tests/cases/markdown_inline_preservation_paragraph.case`: `markdown_inline_preservation_paragraph`              | [pr6-882](pr6/mapping.md#pr6-882) |
| #6  | `tests/cases/markdown_inline_preservation_sentence.case`: `markdown_inline_preservation_sentence`                | [pr6-883](pr6/mapping.md#pr6-883) |
| #4  | `tests/spec_cli.rs`: `markdown_braced_template_spans_allow_heading_formatting`                                   | [pr4-456](pr4/mapping.md#pr4-456) |
| #4  | `tests/spec_cli.rs`: `markdown_hugo_shortcode_tags_preserve_only_the_tag`                                        | [pr4-457](pr4/mapping.md#pr4-457) |

## Validation

The original authored corpus first failed on main at the shortcode-body expectation. The active current-output corpus passes `cargo test --test cli_cases`. All source expectations were separately replayed against both the recorded main executable and the appropriate old-head executable. No unrelated expectations were regenerated.

The unchanged `scripts/check.sh` passed: format checking, Clippy with warnings denied, Rust tests, 1,493 external tests, and 57 editor tests. The two recovered matrix suites also passed separately (1,340 parameter groups). The scope audit verified production files, dependencies, CI, the harness, and every pre-existing `.case` file against the recorded base. Only new tests and supporting test documentation changed.

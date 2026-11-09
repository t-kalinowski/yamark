# Architecture

Yamark has one formatting engine in one Cargo package. The CLI, Git filter, Python distribution, and editor extension share that engine. Public behavior is specified by `website/` and public acceptance tests; this page describes the implementation responsibilities and invariants.

## Ownership

| Owner                                                                                 | Responsibility                                                           |
| ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `src/cli.rs` and `src/cli/`                                                           | Arguments, command dispatch, stdin, and Git filter integration.          |
| `src/workspace.rs` and `src/workspace/`                                               | File traversal, config discovery, file writes, and diff presentation.    |
| `src/config.rs`                                                                       | Configuration parsing, path overrides, and template delimiters.          |
| `src/core/source.rs`                                                                  | Source storage, byte spans, and authored line endings.                   |
| `src/core/document.rs` and `src/core/parser.rs`                                       | Document model, parse/prepare/emit orchestration, and verification.      |
| `src/core/directives.rs`                                                              | Directive parsing, scopes, and effective formatting state.               |
| `src/core/markdown.rs` and `src/core/markdown/`                                       | Markdown block recognition, recursive children, and prepared emission.   |
| `src/core/wrap.rs` and `src/core/wrap/`                                               | Inline recognition, retained literals, wrapping, tables, and containers. |
| `src/core/yaml.rs` and `src/core/yaml/`                                               | YAML parsing, formatting plans, and block/flow emission.                 |
| `src/core/yaml_model.rs`, `yaml_scan.rs`, `yaml_scalar.rs`, and `yaml_equivalence.rs` | YAML representation, scanning, scalar semantics, and verification.       |
| `src/core/source_lang.rs`                                                             | Marked Markdown and YAML inside Python and R source.                     |
| `src/core/emit.rs`                                                                    | Shared tree emission, recursive formatting, and output cleanup.          |
| `src/plugins/`                                                                        | Embedded formatter selection and subprocess invocation.                  |
| `src/json_to_yaml.rs` and `src/json5_to_yaml.rs`                                      | Explicit JSON-family conversion, separate from source formatting.        |
| `editors/vscode/`                                                                     | Editor commands, process invocation, and formatting integration.         |

The private child modules separate responsibilities without changing the public entry points. Keep those boundaries narrow; extraction alone does not justify a new public API or Cargo crate.

## Source and formatting state

Parsing retains source spans and line endings. Prepared output must respect the effective directive state, including file-scoped directives discovered after an earlier node. Recursive Markdown fences, front matter, YAML scalars, and marked source-language regions use the same formatting pipeline.

Explicit preservation and recognized inline literals carry protected byte ranges through nested output. Surrounding editable text still receives normal cleanup. Automatic fallback for unsupported syntax has a different cleanup contract; see [Inline literal preservation](inline-literals.md). Do not broaden a byte-preservation claim to every unsupported construct.

Wrapping must preserve Markdown structure as well as fit the requested width. Moving a marker to a new line can create a list, heading, or other block. Literal and link boundaries must remain intact when planning breaks or restoring container prefixes.

YAML formatting retains unsupported source and checks scalar semantics when choosing spelling, quoting, folding, or collection layout. `--verify` reparses formatted output and checks the supported equivalence contract. It complements exact transcripts and external parser comparisons; it does not replace them.

Preserved YAML children retain their source prefixes. Planning propagates that indentation constraint through containing block collections, and emission keeps those collections at their source indentation while formatting editable siblings. Inline sequence mappings retain the corresponding marker-to-key spacing. Newly emitted literal scalar bodies use the emitted header's indentation rather than the original scalar's indentation.

Preserving a mapping value or sequence item includes its parent entry header, even when the scalar starts on a later line. Ordinary out-of-line scalars retain that layout when parent properties or comments would be lost by moving the value inline. Tags and anchors belong to the scalar's semantic model regardless of which line contains them; their source spans remain separate from the scalar content used for verification.

Compact sequence ancestors can share a physical header with a preserved child. Emission records the ancestor prefix's output boundary so copying that header replaces the already-emitted markers once; later siblings keep their own output. Explicitly formatted scalars retain parent comments and intervening trivia in place, with the same container indentation constraint as preserved values.

Markdown block scalars are parsed when their Markdown target becomes known, including tags attached from a parent line. Their owned source removes YAML indentation before Markdown parsing and retains an origin for diagnostic coordinates. Explicit indentation indicators use the enclosing YAML parent's indentation, including when the header starts on a later line. Emission restores YAML indentation around the formatted Markdown; byte spans and line indexes remain local to the owned source.

Ordinary Markdown wrapping plans are created and resolved one node at a time after directive policy settles. Explicit Markdown targets still prepare during parsing to retain diagnostic and skip precedence. Validation keeps the parsed structure without allocating ordinary emission drafts. Scalar spelling alternatives share a temporary decoded value, and wrapped YAML lines borrow slices of that value rather than copying each line.

Block scalar comments are content only at or beyond the explicit or inferred content indentation. A dedented comment ends the body and remains available to YAML directive processing. When a YAML comment follows a scalar, its content and containing collections retain their source indentation so formatting cannot absorb that comment into the value. Blank embedded output followed by a comment indented beyond the YAML parent is rejected: there is no content left to establish that boundary, and changing the comment's indentation or the embedded target's scalar style is unsupported.

Ordinary conversion from plain or quoted scalars to block style checks the first following nonblank line against the emitted body indentation. If that line is a comment at or beyond the body indentation, the scalar remains plain or quoted. This check runs only for eligible block layouts and uses the source line index without allocating another buffer. Flow expansion carries the enclosing collection's following comment boundary through its final descendants, since emission removes the closing delimiters.

Explicit inline Markdown targets still format when followed by an indented comment. Their new block content uses indentation beyond the comment, keeping the comment outside the value. Empty targets remain quoted empty strings.

Retained quoted continuations keep their source indentation when the parent moves left. YAML quoted line folding discards that physical indentation; retaining it accommodates continuations left of the opening quote and keeps indented document-marker text inside the scalar. Moving the parent right still shifts the continuation lines with it.

## Markdown list recognition and layout

Paragraph boundaries follow Pandoc's single-line setext heading rule. Once a paragraph has begun, a later text/underline pair and thematic-break markers remain paragraph text. The underline check precedes generic raw-block and list recognition so those recognizers cannot promote a continuation into a new block. Fences still interrupt paragraphs to match knitr chunk detection.

`markdown/list.rs` owns the authored structure inside a recognized list span. Its compact item vector records parent IDs, marker and checkbox spans, source indentation, and content offsets. Its ordered block vector records paragraph spans, blank runs, and recursive child blocks with their owning item. These spans are relative to the enclosing list slice; public document-node spans remain absolute in their source buffer.

`markdown/containers.rs` owns the outer list boundary and conservative capability checks. Those checks consult the recorded item's indentation instead of reconstructing the current item from each line. Recognition of a child is not itself permission to rewrite its enclosing list: mixed raw regions, shifted siblings, child headings, and ambiguous ancestor continuations retain their existing preservation boundaries.

`markdown/planning.rs` retains the list structure until effective directives settle. Ordinary lists defer inline drafts until finalization, which resolves one document node at a time; explicit Markdown targets still prepare early to preserve diagnostic precedence. A late layout or template policy can rebuild the draft from the retained structure without recognizing list ownership again. The structure is released during finalization, before emission.

`wrap/containers.rs` consumes these records to construct paragraph prefixes and recursive child plans. It does not split raw text into independent simple/rich lists. A paragraph after a fence still belongs to its recorded item, and nested continuation lines use the recorded nested owner. Authored child indentation is retained independently of normalized output marker spacing. Layout declines a rewrite if shortening a marker would move a child across the four-space indented-code boundary, or would leave a fenced div off the item's content column, where Pandoc no longer recognizes it. A fenced div child extends to its matching closing fence, including blank lines, and a grid table border starts a child block. Checkbox text affects prose alignment, but never the structural content column used to recognize children. Structural prefix slicing counts authored ASCII spaces; Unicode whitespace in content cannot be turned into a byte offset on another line.

The public standalone `format_markdown_list` helper uses the same recognizer and layout implementation. Other container kinds retain their existing paths. This is a bounded list refactor, not a new generic Markdown syntax tree or a promise that each byte is visited once: outer-boundary discovery, structural recognition, capability checks, and inline layout remain distinct walks. Source spans and per-block planning keep the representation compact; the existing allocation and peak-memory checks protect the ordinary Markdown path.

When extending list children, define their source boundary and item ownership in `markdown/list.rs`, then check whether the conservative capability rules permit the rewrite. Reuse the generic child-plan path where possible. Add public examples that prove useful formatting and preservation boundaries, including the construct before and after another child. The [Markdown development group](DEVELOPMENT.md#validation) is the focused check; the full gate still applies before handoff.

With `format-trace` enabled, diagnostics inspect finalized retained preservation and layout decisions. A nominal formatting node can still have no usable plan; that is reported as skipped. Explicitly preserved parents and skipped documents do not produce success messages for descendants that emission will never enter. Tracing does not run a second recognizer.

## YAML parsing and planning

`YamlParser` owns the input cursor, trivia, directive engine, and construction of semantic nodes. `YamlPlanner` in `yaml/plan.rs` borrows the source, effective options, document, and YAML arena to select emission plans. It has no input cursor or directive engine. Replanning an existing document uses this borrowed view directly; it does not construct a parser or ingest the YAML again.

These are ownership boundaries, not additional physical passes. Parsing still invokes planning at the existing node-completion points. Constructing a planner only borrows existing storage; it does not allocate a second representation or scan the source. `YamlTemplatePresence` retains conservative source/configuration facts and per-state memoization across these short-lived planner calls.

Planning is not a single irreversible decision at node creation:

1. A completed node can receive a provisional plan before its parent's context is known.
2. File-scoped directives patch existing states and nested Markdown documents. A layout-only refresh retains preservation decisions but clears width measurements and recomputes all emission plans. Changes to preservation, target eligibility, or template delimiters also invalidate preservation decisions. Both refreshes retain parser-required parent-entry boundaries and source indentation.
3. Final document planning uses child-before-parent arena order to propagate source-indentation constraints, then applies parent indentation and available width to descendants. Root document markers are parser-owned source structure, not layout decisions.

Do not move work across these boundaries without checking directive error precedence, owned-source diagnostic coordinates, and parent-header preservation. A validation parse still builds semantic structure without ordinary emission planning. This separation preserves the current schedule; it is not a claim that every source byte or AST node is visited only once.

`YamlAstNode::must_preserve_source` is a memo for the current policy, not a durable source fact. The parser records a required parent-entry boundary separately, so a later template directive can invalidate the memo without dropping a tag, anchor, comment, or intervening trivia. Attaching scalar properties resets that structural requirement before the enclosing parser reestablishes it: the new tag can make the scalar an explicit Markdown target. The early/late policy tests in `tests/yaml_policy_invalidation.rs` protect this distinction. Template recognition still follows the existing YAML preservation policy, including whole-scalar preservation for templated Markdown targets.

`YamlPlanRefresh` classifies every `DirectiveDelta` field explicitly, so adding a field requires revisiting cache dependencies. A layout-only delta must not skip emission planning: compactness or Markdown options can change an emission plan while its preservation answer remains valid. Nested-document replanning uses the same classification and refreshes layout even for an empty delta, since its base options may have changed. Property attachment still performs full invalidation because the node's source properties, not just its directive state, changed.

Ordinary and compact block sequences share `parse_sequence_item_at` in `yaml/parse.rs`. That method owns value-recognition precedence, same-line directive handling, node properties, and parent-header preservation. The two sequence loops retain their distinct marker and termination rules, including compact entries that begin after another marker on the same physical line. New sequence value syntax belongs in the shared method, not in two parallel branches.

`tests/yaml_planning_contracts.rs` exercises these boundaries through public parsing, preparation, and formatting APIs. Start with that suite and a focused [CLI transcript](../tests/cases/README.md), then run the full public and allocation suites. The trace assertions cover input ingestion counts; they do not count every planning visit.

## Process and file boundaries

Workspace traversal owns configuration discovery and writes. Format stdin uses its virtual path for file kind and configuration without reading that path. The `to-yaml` command converts stdin according to its JSON-family suffix.

The plugin registry selects embedded formatters. External process code owns stdin/stdout exchange and diagnostics; recursive emitters own dedenting, reindenting, and restoring structural delimiters. Keep these responsibilities separate when changing an external formatter contract.

The editor extension passes text to the CLI and applies returned edits. It does not maintain an independent Markdown or YAML formatter.

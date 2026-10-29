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

Markdown block scalars are prepared when their Markdown target becomes known, including tags attached from a parent line. Their owned source removes YAML indentation before Markdown parsing and retains an origin for diagnostic coordinates. Emission restores YAML indentation around the formatted Markdown; byte spans and line indexes remain local to the owned source.

## Process and file boundaries

Workspace traversal owns configuration discovery and writes. Format stdin uses its virtual path for file kind and configuration without reading that path. The `to-yaml` command converts stdin according to its JSON-family suffix.

The plugin registry selects embedded formatters. External process code owns stdin/stdout exchange and diagnostics; recursive emitters own dedenting, reindenting, and restoring structural delimiters. Keep these responsibilities separate when changing an external formatter contract.

The editor extension passes text to the CLI and applies returned edits. It does not maintain an independent Markdown or YAML formatter.

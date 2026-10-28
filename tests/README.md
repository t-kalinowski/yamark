# Tests

Test observable behavior through the CLI or public library API. Start fixes with a failing regression; establish an existing baseline before mechanical moves.

| Suite                                              | Contract                                                                                                                                  |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| [`cases/`](cases/README.md), run by `cli_cases.rs` | Exact invocation, input, stdout, stderr, and exit status. Every CLI behavior change needs a case.                                         |
| `spec_cli/`                                        | Public CLI specifications grouped by commands, workspace/config, Markdown, YAML, front matter, source languages, and external formatters. |
| Other `tests/*.rs`                                 | Focused public API/CLI regressions, packaging, tooling, and documentation checks.                                                         |
| [`external-tests/`](../external-tests/run.py)      | Public CLI, external parser semantics, corpus, smoke, and snapshot comparisons.                                                           |
| [`editors/vscode/test/`](../editors/vscode/test/)  | Editor-facing commands and process integration.                                                                                           |
| `*_performance.rs` and allocation tests            | Scaling and allocation contracts reached through public APIs.                                                                             |
| [`yaml-test-suite/`](yaml-test-suite/README.md)    | Optional YAML corpus round trips.                                                                                                         |

Editor-only regressions use public editor commands or providers. Control pending formatter promises explicitly when testing document edits, selections, or cancellation during formatting; these states are outside the CLI contract.

The Cargo target remains `spec_cli`; select a contract or case with a filter:

```sh
uv run --no-project --with ruff --with py-yaml12 -- cargo test --test spec_cli yaml_collections
cargo test --test cli_cases
```

Keep test bodies near their owning contract. `spec_cli/support.rs` contains only shared CLI invocation helpers; formatter and filesystem fixtures stay with the suites that use them. Do not add internal production visibility for tests.

Review transcript expectations explicitly. Do not regenerate unrelated cases or normalize away complete diagnostics, meaningful ordering, or newline bytes. Fixture paths under `tests/fixtures/` have Git attributes for byte-sensitive data.

Use temporary repositories and directories for writes. Preserve the caller's `HOME` and tool environment. Capability-specific cases must explain and exercise the difference; shared behavior should use shared cases. See [Development](../docs/DEVELOPMENT.md) for prerequisites and the final gate.

# VS Code Quarto Formatter Limitations

This note summarizes what we learned while testing whether the Yamark VS Code extension can invoke or chain the formatter implemented by the Quarto VS Code extension.

## Short Version

Yamark cannot reliably invoke the Quarto VS Code document formatter while Yamark is the configured default formatter for `.qmd` files.

The Quarto extension does not expose a public `quarto.formatDocument` command or standalone formatter binary. Its formatter behavior is implemented as VS Code formatter middleware. VS Code's public formatter command, `vscode.executeFormatDocumentProvider`, does not let an extension choose a specific provider such as `quarto.quarto`; it follows VS Code formatter selection rules.

For users who switch `.qmd` format-on-save from Quarto to Yamark, the main loss is Quarto's cell-aware formatter bridge for executable code cells. They still keep other Quarto extension features such as preview, render, cell execution, syntax highlighting, diagnostics, completions, hover help, visual/source mode, and render-on-save.

## Quarto Formatter Behavior

The installed Quarto extension version inspected was `quarto.quarto-1.132.0`.

Relevant command surface:

- Quarto exposes `quarto.formatCell`.
- Quarto does not expose a `quarto.formatDocument` command.
- Quarto registers document and range formatting through language-client middleware, not through a directly callable command.

The formatter implementation is active-editor dependent:

- It checks `vscode.window.activeTextEditor`.
- It returns no edits if the active editor is missing or does not match the requested document.
- It parses the active Quarto document and uses the active selection line to find a code cell.
- `quarto.formatCell` formats the current cell and applies edits directly to the editor.
- Document formatting delegates code-cell formatting to other VS Code formatter providers by creating virtual documents for cell contents and calling `vscode.executeFormatDocumentProvider` on those virtual documents.

That design is valid for Quarto as a first-class VS Code formatter, but it is a poor API for another extension to call as a chain step.

## What Quarto Formatting Provides

When Quarto's formatter is selected and usable, it can provide behavior that Yamark does not get by invoking the Quarto CLI:

- It can format executable cells through installed VS Code language formatters.
- It maps virtual code-cell edits back into the `.qmd` document.
- It handles Quarto cell option directives such as `#| label: ...` so language formatters do not rewrite or reflow them.
- It has safeguards around cell edit ranges.
- It can format the current cell through `quarto.formatCell`.

In the experiment, Quarto's formatter path formatted a Python executable cell through a virtual `.py` document. Yamark correctly skipped the Quarto `.vdoc.*.py` temp document to avoid recursion.

## What Users Lose If Yamark Replaces Quarto Format-On-Save

If users set `.qmd` format-on-save to Yamark instead of Quarto, they lose Quarto's formatter bridge for `.qmd` files:

- No Quarto-driven formatting of executable code cells.
- No Quarto-managed preservation of `#|` cell directives during code-cell formatting.
- No Quarto mapping of virtual cell formatter edits back into the source `.qmd`.
- No Quarto range/cell formatter behavior through normal format-on-save.
- No active-cell-aware Quarto formatter behavior.

They do not lose Quarto extension features unrelated to being the default formatter:

- Preview and render commands.
- Render-on-save.
- Code cell execution.
- Cell navigation.
- Visual/source mode.
- Syntax highlighting.
- Diagnostics, completions, document symbols, links, folding, hover help, and signature help.
- Math, diagram, and assist previews.

Yamark can replace some formatter behavior through its own embedded formatter configuration or stdin/stdout chaining, for example Ruff or Air. That is Yamark's formatting model, not Quarto's formatter model.

## Experiments

The experiments were run in:

```text
/tmp/yamark-vscode-chain.4h8PHR
```

The test document was `actual-quarto.qmd`. It included:

- YAML front matter.
- A long markdown paragraph.
- Python and R executable cells.
- Fenced markdown, JSON, JSONC, JSON5, GraphQL, CSS, SCSS, LESS, PostCSS, HTML, JS, JSX, TS, and TSX blocks.

### Save With Yamark As Default Formatter

Configuration:

```json
"[quarto]": {
  "editor.defaultFormatter": "yamark.yamark",
  "editor.formatOnSave": true,
  "yamark.nextFormatterExecutable": [],
  "yamark.runNextFormatter": false
}
```

An experimental Yamark build attempted to:

1. Run on save as the `.qmd` default formatter.
2. Temporarily suppress/unregister Yamark's provider.
3. Call `vscode.executeFormatDocumentProvider`.
4. Run Yamark on the result.

Observed result:

- Yamark was invoked on save.
- Yamark temporarily disposed its formatter provider.
- `vscode.executeFormatDocumentProvider` returned no edits.
- Quarto did not receive a formatting request.
- The file was saved without Quarto formatting.

Key Yamark log facts:

```text
formatter step=start name=vscode-format-providers
provider registration action=dispose reason=execute-vscode-format-providers
formatter step=error name=vscode-format-providers err=Yamark: vscode.executeFormatDocumentProvider did not return edits
```

Interpretation:

VS Code did not fall through from the configured default formatter, `yamark.yamark`, to `quarto.quarto` after Yamark temporarily unregistered itself. This makes the nested-provider approach invalid for normal save-time chaining.

### Save With Quarto As Default Formatter

Configuration was changed so `[quarto].editor.defaultFormatter` was `quarto.quarto`.

Observed result:

- Yamark was not invoked, as expected.
- The save command completed.
- The Quarto logs did not show a document formatting request.
- The file was saved without formatter changes in that automated save run.

This confirmed the basic VS Code constraint: one formatter is selected for format-on-save. Setting Quarto as default does not provide a way to run Yamark afterward.

### Direct Provider Invocation

A separate runner extension called:

```js
vscode.commands.executeCommand(
  "vscode.executeFormatDocumentProvider",
  document.uri,
  { tabSize: 2, insertSpaces: true },
);
```

Observed result:

- VS Code invoked Yamark's provider.
- Yamark temporarily unregistered itself.
- The nested formatter-provider call then returned Quarto edits.
- Yamark ran afterward with `--skip-embedded-formatters`.
- The final captured result included Quarto-style cell edits and Yamark document edits.

Key Yamark log facts:

```text
formatter step=end name=vscode-format-providers edits=12 changed=true
formatter step=start name=yamark args=["format","--skip-embedded-formatters","--stdin-file-path", ".../actual-quarto.qmd"]
formatter step=end name=yamark edits=1 changed=true
```

A nested Quarto virtual Python document was also seen:

```text
document uri=file:///tmp/yamark-vscode-chain.4h8PHR/.vdoc....py languageId=python
skipped: Quarto vdoc temp file
```

The direct-provider result proved that Quarto's formatter can be reached in a controlled command context, but it did not prove a shippable save-time chain. The save-time case is governed by default formatter selection and did not work with Yamark as the default formatter.

### Format Cell Command

The runner also called:

```js
vscode.commands.executeCommand("quarto.formatCell");
```

Observed result:

- It formatted the active Python cell.
- It did not format the whole document.
- It applied edits directly to the editor rather than returning a text-edit result to compose with Yamark.

This command is useful for interactive Quarto editing, but it is not a suitable document-formatting chain API.

## Shipping Implications

The current Quarto extension API does not provide a clean way for Yamark to invoke Quarto's document formatter as a chain step.

The following approaches are not good extension-store behavior:

- Temporarily changing the user's `editor.defaultFormatter`.
- Driving `quarto.formatCell` cell-by-cell by moving editor selections.
- Depending on active editor state from inside a document formatter.
- Relying on provider registration timing to force VS Code to select a different formatter.

Reasonable paths are:

- Keep Yamark as the default formatter and use Yamark's own embedded formatter support.
- Keep Yamark as the default formatter and chain stdin/stdout binaries such as Ruff or Air where possible.
- Keep Quarto as the default formatter for users who require Quarto's cell-aware formatting.
- Ask Quarto to expose a public command or extension API that formats a document or returns text edits for a document without depending on active editor state.

## Conclusion

Replacing Quarto format-on-save with Yamark is viable for users who want Yamark's document formatting and can accept Yamark-owned embedded formatter configuration.

It is not equivalent to Quarto format-on-save. The main missing capability is Quarto's VS Code-specific executable-cell formatting bridge.

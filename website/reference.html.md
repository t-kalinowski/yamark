---
title: Reference
description: Find exact details about Yamark's inputs, settings, configuration, directives, and commands.
toc: false
---

Use this section to look up a specific behavior, setting, or syntax. If you are
learning Yamark, start with [Usage](usage.qmd), then work through
[Examples](examples.qmd) and the guided [Directives](directives.qmd) page.

## Find what you need

:::: {.reference-index}
::: {.reference-index-item}
### <span id="file-types"></span><span id="whats-supported"></span><span id="markdown"></span><span id="yaml"></span><span id="source-files"></span><span id="external-formatters"></span><span id="layout-repair"></span><span id="collapse-to-flow-with-or"></span><span id="expand-to-block-with-a-newline"></span><span id="rejection-rules"></span>[Supported files and syntax](reference-files.qmd)

Will Yamark format this file or construct? Check automatically selected and
explicitly marked inputs, preserved syntax, and delegated formatters.
:::

::: {.reference-index-item}
### <span id="safety"></span>[Failures and unchanged content](reference-files.qmd#failure-behavior)

Check which input Yamark rejects, which regions it leaves alone, and what
happens to a file when parsing, formatting, decoding, external formatter
execution, or I/O fails.
:::

::: {.reference-index-item}
### <span id="cli-options"></span><span id="document-markdown-options"></span>[Formatting settings](reference-options.qmd)

Find the control for wrapping, canonical Markdown, footnotes, YAML layout,
widths, indentation, diagnostics, or embedded formatters.
:::

::: {.reference-index-item #configuration}
### <span id="format"></span><span id="template"></span><span id="embedded"></span><span id="paths"></span>[Configuration](reference-config.qmd)

Look up config discovery and the accepted `[format]`, `[template]`,
`[embedded]`, and `[paths]` tables in `yamark.toml`.
:::

::: {.reference-index-item}
### <span id="directives"></span>[Directive syntax](reference-directives.qmd)

Look up the exact `fmt:` comment form, target, scope, accepted values, and
effect for a source-local directive.
:::

::: {.reference-index-item}
### <span id="command-modes"></span>[Command line](cli-help.qmd)

Check write, check, diff, and stdin behavior; output streams and exit status;
or generated help for Yamark, `format`, and the `git-filter` command group.
:::
::::

## Choose a control surface

The same formatting behavior can be available at more than one scope.

| Scope | Surface | Use it for |
| --- | --- | --- |
| One file, region, or target | [`fmt:` directives](reference-directives.qmd) | The most local choice, stored beside the affected content. |
| One Markdown document | [Front-matter options](reference-options.qmd#document-markdown-options) | Markdown behavior that travels with the document. |
| A repository or subtree | [`yamark.toml`](reference-config.qmd) | Shared defaults and path-specific formatter setup. |
| One run | [Command-line flags](cli-help.qmd) | A temporary choice or CI invocation. |

The [Formatting settings](reference-options.qmd) page groups these surfaces by
the output they control, so you can find every place a setting is available.

## Related workflows

- [Editors](editors.qmd) covers editor commands, settings, format-on-save,
  formatter chaining, and logs.
- [Git filter](git-filter.qmd) covers clean and smudge behavior, repository
  setup, adoption, checking, and removal.

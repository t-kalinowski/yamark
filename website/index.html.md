---
title: Yamark
description: A formatter for YAML and Markdown.
toc: false
---

::: {.hero-shell}
::: {.hero-copy}
::: {.hero-kicker}
![](assets/favicon.svg){.hero-mark}
[Beta]{.status-chip}
:::

Yamark is a command-line formatter for YAML and Markdown. It formats YAML front
matter and supported fenced blocks alongside Markdown prose. It can also format
explicitly marked Markdown in YAML, Python, and R files. Optional formatters
such as Ruff, Air, and Prettier handle their matching fenced code blocks when
installed.

::: {.hero-actions}
[Install](usage.qmd#install){.hero-button .primary}
[See examples](examples.qmd){.hero-button}
[Benchmarks](benchmarks.qmd){.hero-button}
:::
:::

::: {.terminal-window}
::: {.terminal-chrome}
<span>uvx yamark format</span>
:::

```sh
$ uvx yamark format config.yaml docs/
format config.yaml
format docs/index.md
format docs/reference.qmd
```
:::
:::

::: {.workflow-strip}
::: {.workflow-item}
**Format**

Rewrite supported files in place.
:::

::: {.workflow-item}
**Check**

Use non-mutating CI and diff modes.
:::

::: {.workflow-item}
**Embed**

Format marked Markdown in YAML, Python, or R.
:::

::: {.workflow-item}
**Fences**

Format supported fenced code blocks.
:::
:::



## A quick example

A Markdown file with YAML front matter. The first pane is the input; the second
is what `yamark format` writes back.



:::: {.before-after}
::: {.before-after-pane #demo-before}
**Before** <label class="softwrap-toggle"><input type="checkbox" id="demo-softwrap-toggle"> soft wrap</label>

```markdown
---
title: Project notes
description: Notes for the next Yamark release.
tags: [yaml,markdown,cli]
---

#   Project notes ##

Yamark formats YAML front matter and the Markdown body in one pass. It wraps long paragraphs, normalizes list indentation, and formats supported fenced code blocks.

-  Run `yamark format` to write changes.
- Run `yamark format --check` in CI.
```

:::

::: {.before-after-pane}
**After**

```markdown
---
title: Project notes
description: Notes for the next Yamark release.
tags: [yaml, markdown, cli]
---

# Project notes

Yamark formats YAML front matter and the Markdown body in one pass. It
wraps long paragraphs, normalizes list indentation, and formats
supported fenced code blocks.

- Run `yamark format` to write changes.
- Run `yamark format --check` in CI.
```
:::
::::

Toggle soft wrap on the Before pane to inspect the input's physical lines.
Yamark wraps YAML and Markdown separately, then writes one combined document.

```{=html}
<script>
(function () {
  var toggle = document.getElementById('demo-softwrap-toggle');
  var pane = document.getElementById('demo-before');
  if (!toggle || !pane) return;
  var apply = function () { pane.classList.toggle('soft-wrap', toggle.checked); };
  toggle.addEventListener('change', apply);
  apply();
})();
</script>
```

::: {.feature-grid}
::: {.feature}
### Mixed documents

Format YAML front matter and the Markdown body with one command.
:::

::: {.feature}
### Rewrap after writing

Edit Markdown prose, YAML descriptions, and front matter without maintaining
line breaks by hand.
:::

::: {.feature}
### Optional verification

Use `--verify` to reparse changed YAML regions and reject invalid or
value-changing output before writing.
:::
:::

## Performance



**Yamark formats a 4 MB Markdown document in 109 ms and a 4 MB YAML file in 69 ms.** The next-fastest tool on each is `dprint-markdown` (349 ms) and `yamlfmt` (187 ms). On a directory of 500 YAML files (50 MB), Yamark finishes in 133 ms; the next-fastest formatter, `deno-fmt`, takes 2.6 s.

The [Benchmarks](benchmarks.qmd) page has the full tables, methodology, checked-in
results, and reproduction commands.

---
title: Yamark
description: An extremely fast formatter for YAML and Markdown, written in Rust.
toc: false
---

::: {.hero-shell}
::: {.hero-copy}
::: {.hero-kicker}
![](assets/favicon.svg){.hero-mark}
[Beta]{.status-chip}
:::

Yamark formats YAML and Markdown, whether they stand alone or live inside
another document or source file. It handles both languages itself and can hand
embedded Python, R, JSON, and web code to Ruff, Air, Prettier, or another
configured formatter.

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
**Dispatch**

Send supported embedded code to its formatter.
:::
:::



## A quick example

YAML and Markdown make a practical collaboration surface for people, agents,
and software. YAML carries fields a program can inspect; Markdown carries
prose, instructions, examples, and code that people and language models can
read. Yamark formats those layers together, much as a code formatter formats
source.

The file below deliberately combines a long YAML scalar, a long Markdown
paragraph, and a nested list. The first pane is the input; the second is what
`yamark format` writes back.



:::: {.before-after}
::: {.before-after-pane #demo-before}
**Before** <label class="softwrap-toggle"><input type="checkbox" id="demo-softwrap-toggle"> soft wrap</label>

```markdown
---
title: Why YAML + Markdown?
description: Structured fields for software alongside prose, instructions, examples, and code that people and language models can read.
tags: [agents,authoring,formats]
---

#   Why YAML + Markdown? ##

YAML front matter and a Markdown body work well for files shared between people, language models, and software. The front matter carries fields a program can inspect; the body carries instructions that people can edit and review in a diff.

This shape is useful for:

- Agent skills and prompt files where metadata sits next to free-form instructions.
  - The body can carry examples and nested Markdown structures.
- Repository documents that need to render for readers and remain easy for tools to inspect.
- Text that moves between code, documentation, and agent context without a separate source format.
```

:::

::: {.before-after-pane}
**After**

```markdown
---
title: Why YAML + Markdown?
description: >-
  Structured fields for software alongside prose, instructions, examples,
  and code that people and language models can read.
tags: [agents, authoring, formats]
---

# Why YAML + Markdown?

YAML front matter and a Markdown body work well for files shared between
people, language models, and software. The front matter carries fields a
program can inspect; the body carries instructions that people can edit
and review in a diff.

This shape is useful for:

- Agent skills and prompt files where metadata sits next to free-form
  instructions.
  - The body can carry examples and nested Markdown structures.
- Repository documents that need to render for readers and remain easy
  for tools to inspect.
- Text that moves between code, documentation, and agent context without
  a separate source format.
```
:::
::::

Toggle soft wrap on the Before pane to inspect the input's physical lines.
Yamark wraps and indents each region according to its own grammar.

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

Edit Markdown prose, YAML descriptions, prompt bodies, and front matter without
maintaining line breaks by hand.
:::

::: {.feature}
### Nested formatters

Format YAML and Markdown recursively, and pass supported embedded source to
Ruff, Air, Prettier, or another configured formatter.
:::
:::

## Performance



**Yamark formats a 4 MB Markdown document in 109 ms and a 4 MB YAML file in 69 ms.** The next-fastest tool on each is `dprint-markdown` (349 ms) and `yamlfmt` (187 ms). On a directory of 500 YAML files (50 MB), Yamark finishes in 133 ms; the next-fastest formatter, `deno-fmt`, takes 2.6 s.

The [Benchmarks](benchmarks.qmd) page has the full tables, methodology, checked-in
results, and reproduction commands.

---
title: Yamark
description: An extremely fast formatter for YAML and Markdown, written in Rust.
toc: false
---

::: {.hero-shell}
::: {.hero-copy}
::: {.hero-kicker}
![](assets/favicon.svg){.hero-mark}
:::

Markdown and YAML are source files too: they hold documentation, configuration,
prompts, and agent instructions. Yamark gives them the consistent formatting we
expect from code, wherever they appear. That includes standalone files, front
matter, fenced blocks, YAML scalars, and marked strings or comments in Python
and R.

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
**Recurse**

Format nested YAML and Markdown inside fenced blocks.
:::
:::



## A quick example

YAML and Markdown often share a file. YAML carries fields software can inspect;
Markdown carries prose, instructions, examples, and code. Yamark formats both
layers together, much as a code formatter formats source.

The file below deliberately combines a long YAML scalar, a long Markdown
paragraph, and a nested list. The first pane is the input; the second is what
`yamark format` writes back.



:::: {.before-after}
::: {.before-after-pane #demo-before}
**Before** <label class="softwrap-toggle"><input type="checkbox" id="demo-softwrap-toggle"> soft wrap</label>

```markdown
---
title: Why YAML + Markdown?
description: Structured fields for software alongside prose, instructions, examples, and code for readers.
tags: [documentation,configuration,formatting]
---

#   Why YAML + Markdown? ##

YAML front matter and a Markdown body let one file serve readers and software. The front matter carries fields a program can inspect; the body carries prose that people can edit and review in a diff.

This shape is useful for:

- Repository documents that need to render for readers and remain easy for tools to inspect.
  - The body can carry examples and nested Markdown structures.
- Prompts and agent instructions where metadata sits next to free-form text.
- Text that moves between documentation, configuration, and tool input without a separate source format.
```

:::

::: {.before-after-pane}
**After**

```markdown
---
title: Why YAML + Markdown?
description: >-
  Structured fields for software alongside prose, instructions, examples,
  and code for readers.
tags: [documentation, configuration, formatting]
---

# Why YAML + Markdown?

YAML front matter and a Markdown body let one file serve readers and
software. The front matter carries fields a program can inspect; the
body carries prose that people can edit and review in a diff.

This shape is useful for:

- Repository documents that need to render for readers and remain easy
  for tools to inspect.
  - The body can carry examples and nested Markdown structures.
- Prompts and agent instructions where metadata sits next to free-form
  text.
- Text that moves between documentation, configuration, and tool input
  without a separate source format.
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
### Nested content

Format YAML and Markdown recursively, and send supported embedded code to its
own formatter when configured.
:::
:::

## Performance



**Yamark formats a 4 MB Markdown document in 109 ms and a 4 MB YAML file in 69 ms.** The next-fastest tool on each is `dprint-markdown` (349 ms) and `yamlfmt` (187 ms). On a directory of 500 YAML files (50 MB), Yamark finishes in 133 ms; the next-fastest formatter, `deno-fmt`, takes 2.6 s.

The [Benchmarks](benchmarks.qmd) page has the full tables, methodology, checked-in
results, and reproduction commands.

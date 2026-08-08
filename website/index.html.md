---
title: Yamark
toc: false
---



::: {.hero-shell}
::: {.hero-copy}
::: {.hero-kicker}
![](assets/favicon.svg){.hero-mark}
:::

<h2 class="hero-thesis">Format Markdown and YAML wherever they live.</h2>

Yamark formats whole files and embedded content with the consistency we expect
from code, keeping source readable and changes easy to review.

::: {.hero-proof}
Fast, deterministic, and written in Rust.
:::

::: {.hero-actions}
[Install](usage.qmd#install){.hero-button .primary}
[See examples](examples.qmd){.hero-button}
:::

::: {.hero-command}
<span>Run directly from PyPI</span>
`uvx yamark format`
:::
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

Here is a Markdown file with YAML front matter, a long paragraph, and a nested
list. Yamark formats the YAML and Markdown in one pass. The first pane shows the
input; the second shows what `yamark format` writes back.



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



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 82–114 ms for each generated 4 MB file and 188 ms for 500 generated YAML files (50 MB).

The [Benchmarks](benchmarks.qmd) page includes the complete comparisons,
methodology, checked-in results, and reproduction commands.

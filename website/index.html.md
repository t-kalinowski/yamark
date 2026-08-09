---
title: Yamark
body-classes: yamark-home
toc: false
---



::: {.hero-shell}
::: {.hero-copy}
::: {.hero-kicker}
![](assets/favicon.svg){.hero-mark}
:::

<h1 class="hero-thesis">Format Markdown and YAML wherever they live.</h1>

Yamark formats whole files and embedded content with the consistency we expect
from code, keeping source readable and changes easy to review.

::: {.hero-proof}
Fast and written in Rust.
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

## Performance



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 82–114 ms for each generated 4 MB file and 188 ms for 500 generated YAML files (50 MB).

<figure class="benchmark-chart benchmark-overview-chart" aria-labelledby="homepage-benchmark-overview-caption">
<figcaption id="homepage-benchmark-overview-caption">
<h3>Yamark and the next-lowest elapsed time</h3>
<p>Median wall time for each generated workload. Lower is better; the horizontal scale is logarithmic.</p>
</figcaption>
<div class="benchmark-chart-canvas" data-benchmark-chart="overview" data-benchmark-source="homepage-benchmark-overview-data"></div>
<div class="benchmark-chart-fallback"><p>Exact values:</p><ul><li><strong>4 MB Markdown:</strong> Yamark 114 ms; dprint 358 ms. Both rewrite Markdown.</li><li><strong>4 MB YAML:</strong> Yamark 82 ms; yamlfmt 194 ms. Both rewrite YAML.</li><li><strong>4 MB Markdown + 200 KB YAML front matter:</strong> Yamark 112 ms; dprint 346 ms. dprint leaves YAML front matter untouched.</li><li><strong>500 YAML files (50 MB):</strong> Yamark 188 ms; Deno 2.7 s. Both rewrite all 500 files.</li></ul></div>
<script type="application/json" id="homepage-benchmark-overview-data">[{"workload_id":"markdown","workload":"4 MB Markdown","short_workload":"4 MB Markdown","yamark_seconds":0.1137662914115936,"yamark_duration":"114 ms","peer_formatter":"dprint","peer_seconds":0.3579473538557068,"peer_duration":"358 ms","output_note":"Both rewrite Markdown"},{"workload_id":"yaml","workload":"4 MB YAML","short_workload":"4 MB YAML","yamark_seconds":0.08179760456550866,"yamark_duration":"82 ms","peer_formatter":"yamlfmt","peer_seconds":0.1940804164623842,"peer_duration":"194 ms","output_note":"Both rewrite YAML"},{"workload_id":"frontmatter","workload":"4 MB Markdown + 200 KB YAML front matter","short_workload":"4 MB Markdown + front matter","yamark_seconds":0.1116253123618662,"yamark_duration":"112 ms","peer_formatter":"dprint","peer_seconds":0.3461023543495685,"peer_duration":"346 ms","output_note":"dprint leaves YAML front matter untouched"},{"workload_id":"directory","workload":"500 YAML files (50 MB)","short_workload":"500 YAML files · 50 MB","yamark_seconds":0.1882577498909086,"yamark_duration":"188 ms","peer_formatter":"Deno","peer_seconds":2.743481750134379,"peer_duration":"2.7 s","output_note":"Both rewrite all 500 files"}]</script>
</figure>

The [Benchmarks](benchmarks.qmd) page includes the complete comparisons,
methodology, checked-in results, and reproduction commands.

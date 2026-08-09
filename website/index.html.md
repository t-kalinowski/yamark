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



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 73–109 ms for each generated 4 MB file and 163 ms for 500 generated YAML files (50 MB).

<figure class="benchmark-chart benchmark-full-field-chart" aria-labelledby="homepage-benchmark-field-caption">
<figcaption id="homepage-benchmark-field-caption">
<h3>Elapsed time across formatter CLIs</h3>
<p>Every formatter in the checked-in comparison. Median wall time; lower is better. All four panels share a logarithmic seconds scale.</p>
</figcaption>
<div class="benchmark-chart-canvas" data-benchmark-chart="full-field" data-benchmark-source="homepage-benchmark-field-data"></div>
<div class="benchmark-chart-fallback"><p>Exact values:</p><ul><li><strong>4 MB Markdown - Yamark:</strong> 103 ms</li><li><strong>4 MB Markdown - dprint:</strong> 323 ms</li><li><strong>4 MB Markdown - Deno:</strong> 355 ms</li><li><strong>4 MB Markdown - Panache:</strong> 385 ms</li><li><strong>4 MB Markdown - Prettier:</strong> 1.6 s</li><li><strong>4 MB Markdown - mdformat:</strong> 2.9 s</li><li><strong>4 MB YAML - Yamark:</strong> 73 ms</li><li><strong>4 MB YAML - yamlfmt:</strong> 179 ms</li><li><strong>4 MB YAML - Deno:</strong> 726 ms</li><li><strong>4 MB YAML - dprint:</strong> 1.1 s</li><li><strong>4 MB YAML - yamlfix:</strong> 7.0 s</li><li><strong>4 MB YAML - Prettier:</strong> 19.4 s (file unchanged)</li><li><strong>4 MB Markdown + front matter - Yamark:</strong> 109 ms (formatted)</li><li><strong>4 MB Markdown + front matter - dprint:</strong> 357 ms (untouched)</li><li><strong>4 MB Markdown + front matter - Deno:</strong> 456 ms (formatted)</li><li><strong>4 MB Markdown + front matter - Prettier:</strong> 1.9 s (formatted)</li><li><strong>4 MB Markdown + front matter - mdformat:</strong> 3.7 s (not preserved)</li><li><strong>4 MB Markdown + front matter - Panache:</strong> 6.3 s (formatted)</li><li><strong>500 YAML files · 50 MB - Yamark:</strong> 163 ms</li><li><strong>500 YAML files · 50 MB - Deno:</strong> 2.3 s</li><li><strong>500 YAML files · 50 MB - dprint:</strong> 2.6 s</li><li><strong>500 YAML files · 50 MB - yamlfmt:</strong> 4.0 s</li><li><strong>500 YAML files · 50 MB - Prettier:</strong> 49.6 s</li><li><strong>500 YAML files · 50 MB - yamlfix:</strong> 179.4 s</li></ul></div>
<script type="application/json" id="homepage-benchmark-field-data">[{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Yamark","seconds":0.1033876454457641,"duration":"103 ms","is_yamark":true,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"dprint","seconds":0.3231573960511014,"duration":"323 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Deno","seconds":0.3551847710041329,"duration":"355 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Panache","seconds":0.3848891875240952,"duration":"385 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Prettier","seconds":1.61442297953181,"duration":"1.6 s","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"mdformat","seconds":2.90329950011801,"duration":"2.9 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Yamark","seconds":0.07255166652612388,"duration":"73 ms","is_yamark":true,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfmt","seconds":0.178913333103992,"duration":"179 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Deno","seconds":0.7262114164186642,"duration":"726 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"dprint","seconds":1.075787103967741,"duration":"1.1 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfix","seconds":7.042518374510109,"duration":"7.0 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Prettier","seconds":19.36223006201908,"duration":"19.4 s","is_yamark":false,"outcome":"file unchanged"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Yamark","seconds":0.1086301665054634,"duration":"109 ms","is_yamark":true,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"dprint","seconds":0.3567442290950567,"duration":"357 ms","is_yamark":false,"outcome":"untouched"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Deno","seconds":0.4564543960150331,"duration":"456 ms","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Prettier","seconds":1.937399312504567,"duration":"1.9 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"mdformat","seconds":3.653730666614138,"duration":"3.7 s","is_yamark":false,"outcome":"not preserved"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Panache","seconds":6.285816541523673,"duration":"6.3 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Yamark","seconds":0.1628786658402532,"duration":"163 ms","is_yamark":true,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Deno","seconds":2.287125375121832,"duration":"2.3 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"dprint","seconds":2.625868167029694,"duration":"2.6 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfmt","seconds":3.993410249939188,"duration":"4.0 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Prettier","seconds":49.631839749868959,"duration":"49.6 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfix","seconds":179.39152237493545,"duration":"179.4 s","is_yamark":false,"outcome":null}]</script>
</figure>

The [Benchmarks](benchmarks.qmd) page includes detailed tables, methodology,
checked-in results, and reproduction commands.

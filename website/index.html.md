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



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 78–115 ms for each generated 4 MB file and 158 ms for 500 generated YAML files (50 MB).

<figure class="benchmark-chart benchmark-full-field-chart" aria-labelledby="homepage-benchmark-field-caption">
<figcaption id="homepage-benchmark-field-caption">
<h3>Elapsed time across formatter CLIs</h3>
<p>Every formatter in the checked-in comparison. Median wall time; lower is better. All four panels share a logarithmic seconds scale.</p>
</figcaption>
<div class="benchmark-chart-canvas" data-benchmark-chart="full-field" data-benchmark-source="homepage-benchmark-field-data"></div>
<div class="benchmark-chart-fallback"><p>Exact values:</p><ul><li><strong>4 MB Markdown - Yamark:</strong> 113 ms</li><li><strong>4 MB Markdown - dprint:</strong> 359 ms</li><li><strong>4 MB Markdown - Panache:</strong> 406 ms</li><li><strong>4 MB Markdown - Deno:</strong> 408 ms</li><li><strong>4 MB Markdown - Prettier:</strong> 1.8 s</li><li><strong>4 MB Markdown - mdformat:</strong> 3.2 s</li><li><strong>4 MB YAML - Yamark:</strong> 78 ms</li><li><strong>4 MB YAML - yamlfmt:</strong> 190 ms</li><li><strong>4 MB YAML - Deno:</strong> 777 ms</li><li><strong>4 MB YAML - dprint:</strong> 1.1 s</li><li><strong>4 MB YAML - yamlfix:</strong> 7.4 s</li><li><strong>4 MB YAML - Prettier:</strong> 20.1 s (file unchanged)</li><li><strong>4 MB Markdown + front matter - Yamark:</strong> 115 ms (formatted)</li><li><strong>4 MB Markdown + front matter - dprint:</strong> 348 ms (untouched)</li><li><strong>4 MB Markdown + front matter - Deno:</strong> 418 ms (formatted)</li><li><strong>4 MB Markdown + front matter - Prettier:</strong> 1.9 s (formatted)</li><li><strong>4 MB Markdown + front matter - mdformat:</strong> 3.8 s (not preserved)</li><li><strong>4 MB Markdown + front matter - Panache:</strong> 6.7 s (formatted)</li><li><strong>500 YAML files · 50 MB - Yamark:</strong> 158 ms</li><li><strong>500 YAML files · 50 MB - Deno:</strong> 2.3 s</li><li><strong>500 YAML files · 50 MB - dprint:</strong> 2.7 s</li><li><strong>500 YAML files · 50 MB - yamlfmt:</strong> 3.9 s</li><li><strong>500 YAML files · 50 MB - Prettier:</strong> 47.4 s</li><li><strong>500 YAML files · 50 MB - yamlfix:</strong> 183.5 s</li></ul></div>
<script type="application/json" id="homepage-benchmark-field-data">[{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Yamark","seconds":0.1126175619428977,"duration":"113 ms","is_yamark":true,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"dprint","seconds":0.3589440209325403,"duration":"359 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Panache","seconds":0.4059670210117474,"duration":"406 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Deno","seconds":0.4075945625081658,"duration":"408 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Prettier","seconds":1.814105249941349,"duration":"1.8 s","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"mdformat","seconds":3.241150583373383,"duration":"3.2 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Yamark","seconds":0.07822704198770225,"duration":"78 ms","is_yamark":true,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfmt","seconds":0.1900328961201012,"duration":"190 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Deno","seconds":0.7769087500637397,"duration":"777 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"dprint","seconds":1.143036437570117,"duration":"1.1 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfix","seconds":7.376165624940768,"duration":"7.4 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Prettier","seconds":20.144398229429498,"duration":"20.1 s","is_yamark":false,"outcome":"file unchanged"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Yamark","seconds":0.1145400630775839,"duration":"115 ms","is_yamark":true,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"dprint","seconds":0.3479785829549655,"duration":"348 ms","is_yamark":false,"outcome":"untouched"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Deno","seconds":0.418059729039669,"duration":"418 ms","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Prettier","seconds":1.90301912499126,"duration":"1.9 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"mdformat","seconds":3.832500186981633,"duration":"3.8 s","is_yamark":false,"outcome":"not preserved"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Panache","seconds":6.664456958067603,"duration":"6.7 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Yamark","seconds":0.1578086251392961,"duration":"158 ms","is_yamark":true,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Deno","seconds":2.251173458993435,"duration":"2.3 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"dprint","seconds":2.652126084081829,"duration":"2.7 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfmt","seconds":3.923564207972959,"duration":"3.9 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Prettier","seconds":47.350564582971856,"duration":"47.4 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfix","seconds":183.51191100012511,"duration":"183.5 s","is_yamark":false,"outcome":null}]</script>
</figure>

The [Benchmarks](benchmarks.qmd) page includes detailed tables, methodology,
checked-in results, and reproduction commands.

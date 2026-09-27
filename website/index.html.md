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

<h1 class="hero-thesis">A fast formatter for YAML and Markdown.</h1>

Yamark formats whole files and embedded content with the consistency we expect
from code, keeping source readable and changes easy to review.

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

A short Markdown file with YAML front matter - a native document format for LLM
work. It combines software-readable structure with natural language for prompts,
skills, reference documents, and generated output. The Before pane shows the
unformatted source; the After pane shows what `yamark format` writes back.



:::: {.before-after}
::: {.before-after-pane #demo-before}
**Before** <label class="softwrap-toggle"><input type="checkbox" id="demo-softwrap-toggle"> soft wrap</label>

```markdown
---
title: Why YAML + Markdown?
description: The source languages of LLM work - structured fields for tools alongside instructions, examples, and context for models.
tags: [llm,authoring,formats]
---

#   Why YAML + Markdown? ##

When LLMs become part of a software system, YAML and Markdown start to feel like programming languages. YAML defines named fields tools can act on; Markdown carries the instructions, examples, and context that shape model behavior. Together they create a shared source file that stays readable to people and models, exposes explicit structure to software, produces clear diffs, and renders directly.

Where the combination shines:

- Agent skills and prompt files keep names, versions, and tool metadata beside the instructions they describe.
  - The Markdown body can carry code samples and nested structures with their own syntax and indentation.
- Reference documents stay readable to authors and models without a separate source format or build step.
- Tool inputs and model outputs survive chat, Git, and documentation pipelines as recognizable text.
- One source can render as HTML or PDF and travel unchanged as agent context.
```

:::

::: {.before-after-pane}
**After**

```markdown
---
title: Why YAML + Markdown?
description: >-
  The source languages of LLM work - structured fields for tools alongside
  instructions, examples, and context for models.
tags: [llm, authoring, formats]
---

# Why YAML + Markdown?

When LLMs become part of a software system, YAML and Markdown start to
feel like programming languages. YAML defines named fields tools can act
on; Markdown carries the instructions, examples, and context that shape
model behavior. Together they create a shared source file that stays
readable to people and models, exposes explicit structure to software,
produces clear diffs, and renders directly.

Where the combination shines:

- Agent skills and prompt files keep names, versions, and tool metadata
  beside the instructions they describe.
  - The Markdown body can carry code samples and nested structures with
    their own syntax and indentation.
- Reference documents stay readable to authors and models without a
  separate source format or build step.
- Tool inputs and model outputs survive chat, Git, and documentation
  pipelines as recognizable text.
- One source can render as HTML or PDF and travel unchanged as agent
  context.
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



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 79–89 ms for each generated 4 MB file and 133 ms for 500 generated YAML files (50 MB).

<figure class="benchmark-chart benchmark-full-field-chart" aria-labelledby="homepage-benchmark-field-caption">
<figcaption id="homepage-benchmark-field-caption">
<h3>Elapsed time across formatter CLIs</h3>
<p>Every formatter in the checked-in comparison. Median wall time; lower is better. All four panels share a logarithmic seconds scale.</p>
</figcaption>
<div class="benchmark-chart-canvas" data-benchmark-chart="full-field" data-benchmark-source="homepage-benchmark-field-data"></div>
<div class="benchmark-chart-fallback"><p>Exact values:</p><ul><li><strong>4 MB Markdown - Yamark:</strong> 89 ms</li><li><strong>4 MB Markdown - dprint:</strong> 327 ms</li><li><strong>4 MB Markdown - Panache:</strong> 398 ms</li><li><strong>4 MB Markdown - Deno:</strong> 407 ms</li><li><strong>4 MB Markdown - Prettier:</strong> 1.5 s</li><li><strong>4 MB Markdown - mdformat:</strong> 2.8 s</li><li><strong>4 MB YAML - Yamark:</strong> 79 ms</li><li><strong>4 MB YAML - yamlfmt:</strong> 184 ms</li><li><strong>4 MB YAML - Deno:</strong> 800 ms</li><li><strong>4 MB YAML - dprint:</strong> 1.0 s</li><li><strong>4 MB YAML - yamlfix:</strong> 7.1 s</li><li><strong>4 MB YAML - Prettier:</strong> 18.7 s (file unchanged)</li><li><strong>4 MB Markdown + front matter - Yamark:</strong> 89 ms (formatted)</li><li><strong>4 MB Markdown + front matter - dprint:</strong> 317 ms (untouched)</li><li><strong>4 MB Markdown + front matter - Deno:</strong> 446 ms (formatted)</li><li><strong>4 MB Markdown + front matter - Prettier:</strong> 1.6 s (formatted)</li><li><strong>4 MB Markdown + front matter - mdformat:</strong> 3.2 s (not preserved)</li><li><strong>4 MB Markdown + front matter - Panache:</strong> 6.8 s (formatted)</li><li><strong>500 YAML files · 50 MB - Yamark:</strong> 133 ms</li><li><strong>500 YAML files · 50 MB - dprint:</strong> 2.6 s</li><li><strong>500 YAML files · 50 MB - Deno:</strong> 3.2 s</li><li><strong>500 YAML files · 50 MB - yamlfmt:</strong> 3.6 s</li><li><strong>500 YAML files · 50 MB - Prettier:</strong> 41.6 s</li><li><strong>500 YAML files · 50 MB - yamlfix:</strong> 175.6 s</li></ul></div>
<script type="application/json" id="homepage-benchmark-field-data">[{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Yamark","seconds":0.08921762500540353,"duration":"89 ms","is_yamark":true,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"dprint","seconds":0.3271169379877392,"duration":"327 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Panache","seconds":0.3975499170192052,"duration":"398 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Deno","seconds":0.4074254375009332,"duration":"407 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Prettier","seconds":1.543326353974408,"duration":"1.5 s","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"mdformat","seconds":2.827889416483231,"duration":"2.8 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Yamark","seconds":0.07946993751102127,"duration":"79 ms","is_yamark":true,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfmt","seconds":0.1837334164883941,"duration":"184 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Deno","seconds":0.8002548539661802,"duration":"800 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"dprint","seconds":1.033272791508352,"duration":"1.0 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfix","seconds":7.096864062506938,"duration":"7.1 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Prettier","seconds":18.672253437485779,"duration":"18.7 s","is_yamark":false,"outcome":"file unchanged"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Yamark","seconds":0.08882020850433037,"duration":"89 ms","is_yamark":true,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"dprint","seconds":0.3168731664773077,"duration":"317 ms","is_yamark":false,"outcome":"untouched"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Deno","seconds":0.4455461875186302,"duration":"446 ms","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Prettier","seconds":1.615801874984754,"duration":"1.6 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"mdformat","seconds":3.224468520493247,"duration":"3.2 s","is_yamark":false,"outcome":"not preserved"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Panache","seconds":6.759372124972288,"duration":"6.8 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Yamark","seconds":0.1325313340057619,"duration":"133 ms","is_yamark":true,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"dprint","seconds":2.590782792016398,"duration":"2.6 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Deno","seconds":3.233484791009687,"duration":"3.2 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfmt","seconds":3.570427665952593,"duration":"3.6 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Prettier","seconds":41.60775416699471,"duration":"41.6 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfix","seconds":175.64967833296396,"duration":"175.6 s","is_yamark":false,"outcome":null}]</script>
</figure>

The [Benchmarks](benchmarks.qmd) page includes detailed tables, methodology,
checked-in results, and reproduction commands.

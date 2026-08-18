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



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 78–117 ms for each generated 4 MB file and 181 ms for 500 generated YAML files (50 MB).

<figure class="benchmark-chart benchmark-full-field-chart" aria-labelledby="homepage-benchmark-field-caption">
<figcaption id="homepage-benchmark-field-caption">
<h3>Elapsed time across formatter CLIs</h3>
<p>Every formatter in the checked-in comparison. Median wall time; lower is better. All four panels share a logarithmic seconds scale.</p>
</figcaption>
<div class="benchmark-chart-canvas" data-benchmark-chart="full-field" data-benchmark-source="homepage-benchmark-field-data"></div>
<div class="benchmark-chart-fallback"><p>Exact values:</p><ul><li><strong>4 MB Markdown - Yamark:</strong> 117 ms</li><li><strong>4 MB Markdown - dprint:</strong> 378 ms</li><li><strong>4 MB Markdown - Deno:</strong> 412 ms</li><li><strong>4 MB Markdown - Panache:</strong> 417 ms</li><li><strong>4 MB Markdown - Prettier:</strong> 1.9 s</li><li><strong>4 MB Markdown - mdformat:</strong> 3.4 s</li><li><strong>4 MB YAML - Yamark:</strong> 78 ms</li><li><strong>4 MB YAML - yamlfmt:</strong> 196 ms</li><li><strong>4 MB YAML - Deno:</strong> 766 ms</li><li><strong>4 MB YAML - dprint:</strong> 1.2 s</li><li><strong>4 MB YAML - yamlfix:</strong> 7.6 s</li><li><strong>4 MB YAML - Prettier:</strong> 20.6 s (file unchanged)</li><li><strong>4 MB Markdown + front matter - Yamark:</strong> 111 ms (formatted)</li><li><strong>4 MB Markdown + front matter - dprint:</strong> 345 ms (untouched)</li><li><strong>4 MB Markdown + front matter - Deno:</strong> 420 ms (formatted)</li><li><strong>4 MB Markdown + front matter - Prettier:</strong> 1.9 s (formatted)</li><li><strong>4 MB Markdown + front matter - mdformat:</strong> 3.8 s (not preserved)</li><li><strong>4 MB Markdown + front matter - Panache:</strong> 6.5 s (formatted)</li><li><strong>500 YAML files · 50 MB - Yamark:</strong> 181 ms</li><li><strong>500 YAML files · 50 MB - Deno:</strong> 2.5 s</li><li><strong>500 YAML files · 50 MB - dprint:</strong> 3.0 s</li><li><strong>500 YAML files · 50 MB - yamlfmt:</strong> 4.0 s</li><li><strong>500 YAML files · 50 MB - Prettier:</strong> 45.0 s</li><li><strong>500 YAML files · 50 MB - yamlfix:</strong> 176.3 s</li></ul></div>
<script type="application/json" id="homepage-benchmark-field-data">[{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Yamark","seconds":0.1168582085520029,"duration":"117 ms","is_yamark":true,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"dprint","seconds":0.3777924790047109,"duration":"378 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Deno","seconds":0.4124717705417424,"duration":"412 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Panache","seconds":0.4173165620304644,"duration":"417 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Prettier","seconds":1.886349040898494,"duration":"1.9 s","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"mdformat","seconds":3.358799333451316,"duration":"3.4 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Yamark","seconds":0.07816008350346237,"duration":"78 ms","is_yamark":true,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfmt","seconds":0.1964544999646023,"duration":"196 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Deno","seconds":0.7658654995029792,"duration":"766 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"dprint","seconds":1.1645917709684,"duration":"1.2 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfix","seconds":7.550535458605736,"duration":"7.6 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Prettier","seconds":20.620182874961756,"duration":"20.6 s","is_yamark":false,"outcome":"file unchanged"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Yamark","seconds":0.110607854090631,"duration":"111 ms","is_yamark":true,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"dprint","seconds":0.3445675210095942,"duration":"345 ms","is_yamark":false,"outcome":"untouched"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Deno","seconds":0.4197737494250759,"duration":"420 ms","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Prettier","seconds":1.85313666658476,"duration":"1.9 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"mdformat","seconds":3.774733791477047,"duration":"3.8 s","is_yamark":false,"outcome":"not preserved"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Panache","seconds":6.544588541495614,"duration":"6.5 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Yamark","seconds":0.1807032499928027,"duration":"181 ms","is_yamark":true,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Deno","seconds":2.478343124967068,"duration":"2.5 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"dprint","seconds":2.958234000019729,"duration":"3.0 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfmt","seconds":4.031520792050287,"duration":"4.0 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Prettier","seconds":44.952759457984939,"duration":"45.0 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfix","seconds":176.27065745787695,"duration":"176.3 s","is_yamark":false,"outcome":null}]</script>
</figure>

The [Benchmarks](benchmarks.qmd) page includes detailed tables, methodology,
checked-in results, and reproduction commands.

---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 78–117 ms for each generated 4 MB file and 181 ms for 500 generated YAML files (50 MB).

Each table compares the formatter CLIs in this harness that accept that input.
Tools use their default formatting behavior, with no formatting options, shims,
or adapters, so the roster differs by input kind.

## At a glance

The next-lowest result is the lowest median wall time among the other CLIs for
that workload. Relative time shows that result as a multiple of Yamark's time.
The output note keeps differences in default formatting behavior beside the
timing.

::: {.benchmark-summary-shell}
<table class="perf-table benchmark-summary-table">
<caption>Yamark and the next-lowest elapsed time in each workload</caption>
 <thead>
  <tr>
   <th style="text-align:left;"> Workload </th>
   <th style="text-align:right;"> Yamark </th>
   <th style="text-align:left;"> Next-lowest elapsed </th>
   <th style="text-align:right;"> Relative time </th>
   <th style="text-align:left;"> Output note </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> 4 MB Markdown </td>
   <td style="text-align:right;"> 117 ms </td>
   <td style="text-align:left;"> dprint · 378 ms </td>
   <td style="text-align:right;"> 3.2× </td>
   <td style="text-align:left;"> Both rewrite Markdown </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 4 MB YAML </td>
   <td style="text-align:right;"> 78 ms </td>
   <td style="text-align:left;"> yamlfmt · 196 ms </td>
   <td style="text-align:right;"> 2.5× </td>
   <td style="text-align:left;"> Both rewrite YAML </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 4 MB Markdown + 200 KB YAML front matter </td>
   <td style="text-align:right;"> 111 ms </td>
   <td style="text-align:left;"> dprint · 345 ms </td>
   <td style="text-align:right;"> 3.1× </td>
   <td style="text-align:left;"> dprint leaves YAML front matter untouched </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 500 YAML files (50 MB) </td>
   <td style="text-align:right;"> 181 ms </td>
   <td style="text-align:left;"> Deno · 2.5 s </td>
   <td style="text-align:right;"> 13.7× </td>
   <td style="text-align:left;"> Both rewrite all 500 files </td>
  </tr>
</tbody>
</table>
:::

<figure class="benchmark-chart benchmark-full-field-chart" aria-labelledby="benchmark-full-field-caption">
<figcaption id="benchmark-full-field-caption">
<h3>Elapsed time by formatter</h3>
<p>Median wall time; lower is better. All four panels share a logarithmic seconds scale.</p>
</figcaption>
<div class="benchmark-chart-canvas" data-benchmark-chart="full-field" data-benchmark-source="benchmark-full-field-data"></div>
<div class="benchmark-chart-fallback"><p>Exact values:</p><ul><li><strong>4 MB Markdown - Yamark:</strong> 117 ms</li><li><strong>4 MB Markdown - dprint:</strong> 378 ms</li><li><strong>4 MB Markdown - Deno:</strong> 412 ms</li><li><strong>4 MB Markdown - Panache:</strong> 417 ms</li><li><strong>4 MB Markdown - Prettier:</strong> 1.9 s</li><li><strong>4 MB Markdown - mdformat:</strong> 3.4 s</li><li><strong>4 MB YAML - Yamark:</strong> 78 ms</li><li><strong>4 MB YAML - yamlfmt:</strong> 196 ms</li><li><strong>4 MB YAML - Deno:</strong> 766 ms</li><li><strong>4 MB YAML - dprint:</strong> 1.2 s</li><li><strong>4 MB YAML - yamlfix:</strong> 7.6 s</li><li><strong>4 MB YAML - Prettier:</strong> 20.6 s (file unchanged)</li><li><strong>4 MB Markdown + front matter - Yamark:</strong> 111 ms (formatted)</li><li><strong>4 MB Markdown + front matter - dprint:</strong> 345 ms (untouched)</li><li><strong>4 MB Markdown + front matter - Deno:</strong> 420 ms (formatted)</li><li><strong>4 MB Markdown + front matter - Prettier:</strong> 1.9 s (formatted)</li><li><strong>4 MB Markdown + front matter - mdformat:</strong> 3.8 s (not preserved)</li><li><strong>4 MB Markdown + front matter - Panache:</strong> 6.5 s (formatted)</li><li><strong>500 YAML files · 50 MB - Yamark:</strong> 181 ms</li><li><strong>500 YAML files · 50 MB - Deno:</strong> 2.5 s</li><li><strong>500 YAML files · 50 MB - dprint:</strong> 3.0 s</li><li><strong>500 YAML files · 50 MB - yamlfmt:</strong> 4.0 s</li><li><strong>500 YAML files · 50 MB - Prettier:</strong> 45.0 s</li><li><strong>500 YAML files · 50 MB - yamlfix:</strong> 176.3 s</li></ul></div>
<script type="application/json" id="benchmark-full-field-data">[{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Yamark","seconds":0.1168582085520029,"duration":"117 ms","is_yamark":true,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"dprint","seconds":0.3777924790047109,"duration":"378 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Deno","seconds":0.4124717705417424,"duration":"412 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Panache","seconds":0.4173165620304644,"duration":"417 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Prettier","seconds":1.886349040898494,"duration":"1.9 s","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"mdformat","seconds":3.358799333451316,"duration":"3.4 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Yamark","seconds":0.07816008350346237,"duration":"78 ms","is_yamark":true,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfmt","seconds":0.1964544999646023,"duration":"196 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Deno","seconds":0.7658654995029792,"duration":"766 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"dprint","seconds":1.1645917709684,"duration":"1.2 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfix","seconds":7.550535458605736,"duration":"7.6 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Prettier","seconds":20.620182874961756,"duration":"20.6 s","is_yamark":false,"outcome":"file unchanged"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Yamark","seconds":0.110607854090631,"duration":"111 ms","is_yamark":true,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"dprint","seconds":0.3445675210095942,"duration":"345 ms","is_yamark":false,"outcome":"untouched"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Deno","seconds":0.4197737494250759,"duration":"420 ms","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Prettier","seconds":1.85313666658476,"duration":"1.9 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"mdformat","seconds":3.774733791477047,"duration":"3.8 s","is_yamark":false,"outcome":"not preserved"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Panache","seconds":6.544588541495614,"duration":"6.5 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Yamark","seconds":0.1807032499928027,"duration":"181 ms","is_yamark":true,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Deno","seconds":2.478343124967068,"duration":"2.5 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"dprint","seconds":2.958234000019729,"duration":"3.0 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfmt","seconds":4.031520792050287,"duration":"4.0 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Prettier","seconds":44.952759457984939,"duration":"45.0 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfix","seconds":176.27065745787695,"duration":"176.3 s","is_yamark":false,"outcome":null}]</script>
</figure>

## Detailed results

::: {.panel-tabset}

### Markdown

One generated 4 MB Markdown document (`big.md`): prose paragraphs, links, and
nested lists. Each tool's CLI formats a fresh copy. Wall time includes process
startup and is the median of 10 measured runs after 2 warmup runs. Peak
RSS is the median across measured runs.

<table class="perf-table">
 <thead>
  <tr>
   <th style="text-align:left;"> Formatter </th>
   <th style="text-align:right;"> Wall time </th>
   <th style="text-align:right;"> Peak RSS </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> yamark </td>
   <td style="text-align:right;"> 117 ms </td>
   <td style="text-align:right;"> 12.7 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 378 ms </td>
   <td style="text-align:right;"> 172.8 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 412 ms </td>
   <td style="text-align:right;"> 482.7 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 417 ms </td>
   <td style="text-align:right;"> 32.2 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.9 s </td>
   <td style="text-align:right;"> 596.2 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.4 s </td>
   <td style="text-align:right;"> 152.5 MB </td>
  </tr>
</tbody>
</table>

### YAML

One generated 4 MB YAML file (`big.yaml`): block maps and sequences,
block scalars, and comments. Same procedure as the Markdown comparison.

<table class="perf-table">
 <thead>
  <tr>
   <th style="text-align:left;"> Formatter </th>
   <th style="text-align:right;"> Wall time </th>
   <th style="text-align:right;"> Peak RSS </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> yamark </td>
   <td style="text-align:right;"> 78 ms </td>
   <td style="text-align:right;"> 51.7 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 196 ms </td>
   <td style="text-align:right;"> 302.8 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 766 ms </td>
   <td style="text-align:right;"> 142.0 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 1.2 s </td>
   <td style="text-align:right;"> 118.7 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 7.6 s </td>
   <td style="text-align:right;"> 269.9 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 20.6 s </td>
   <td style="text-align:right;"> 1024.3 MB </td>
  </tr>
</tbody>
</table>

### Markdown + front matter

The same 4 MB document shape (`big-with-frontmatter.md`) with a
200 KB deliberately unformatted YAML
front matter block - about a third of the document's lines are YAML. The
Front matter column reports what each tool did with that block:
**formatted** (rewrote it), **untouched** (passed it through), or **not
preserved** (broke the delimiters).

<table class="perf-table">
 <thead>
  <tr>
   <th style="text-align:left;"> Formatter </th>
   <th style="text-align:right;"> Wall time </th>
   <th style="text-align:right;"> Peak RSS </th>
   <th style="text-align:center;"> Front matter </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> yamark </td>
   <td style="text-align:right;"> 111 ms </td>
   <td style="text-align:right;"> 15.2 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 345 ms </td>
   <td style="text-align:right;"> 173.0 MB </td>
   <td style="text-align:center;"> untouched </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 420 ms </td>
   <td style="text-align:right;"> 585.3 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.9 s </td>
   <td style="text-align:right;"> 585.7 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.8 s </td>
   <td style="text-align:right;"> 180.1 MB </td>
   <td style="text-align:center;"> not preserved </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 6.5 s </td>
   <td style="text-align:right;"> 40.9 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
</tbody>
</table>

The harness derives that column from the output bytes: trailing-whitespace
trimming does not count as formatting. `dprint`'s Markdown plugin passes
front matter through unformatted. `mdformat`, installed without its front-matter
plugin, reads the opening `---` as a thematic break and does not preserve the
front matter delimiters.

### Directory

500 generated YAML service-configuration files
of about 100 KB each (50 MB in
total). Each tool receives the directory root once and discovers the files
itself. Every run uses a fresh copy and is verified to reformat all
500 files. Wall time is the median of 3 measured runs after 1 warmup run.

<table class="perf-table">
 <thead>
  <tr>
   <th style="text-align:left;"> Formatter </th>
   <th style="text-align:right;"> Wall time </th>
   <th style="text-align:right;"> User CPU time </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> yamark </td>
   <td style="text-align:right;"> 0.181 s </td>
   <td style="text-align:right;"> 1.226 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 2.478 s </td>
   <td style="text-align:right;"> 33.541 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 2.958 s </td>
   <td style="text-align:right;"> 39.450 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 4.032 s </td>
   <td style="text-align:right;"> 4.431 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 44.953 s </td>
   <td style="text-align:right;"> 67.189 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 176.271 s </td>
   <td style="text-align:right;"> 175.446 s </td>
  </tr>
</tbody>
</table>

Yamark formats the directory in parallel, as do `deno-fmt` and `dprint`, so
total user CPU time can exceed wall time. User CPU time is summed across threads
and processes and excludes system CPU time.

:::

## Methodology

The benchmark scripts (`tools/bench/big.py` and `tools/bench/run.py`) generate
the corpora deterministically and run each included tool through its own CLI.
Across the four workloads, Yamark is compared with `deno-fmt`, `dprint-markdown`, `dprint-yaml`, `mdformat`, `panache`, `prettier`, `yamlfix`, `yamlfmt`.

Two details keep cached work out of the timings and limit `dprint`'s
configuration to plugin selection:

- `panache` and `dprint` cache formatting results. The harness stores their
  caches in the benchmark work directory and clears cached formatting results
  between repetitions, so each timed run formats from scratch.
- `dprint` has no built-in plugins, so it runs with a config file that
  names the first-party plugin for the input kind and sets no formatting
  options.

Autofixing linters (`pymarkdown`, `markdownlint-cli2`) are outside this
formatter-CLI comparison. The harness can still run them with `--tools`.

Measured on a MacBook Pro (Apple M4 Max, macOS arm64) using the published PyPI release
Yamark `0.3.0`, built from commit
`2518d01011b3`.
Tool versions: `yamark 0.3.0`, `deno 2.9.5 (stable, release, aarch64-apple-darwin)`, `dprint 0.55.2`, `yamlfmt 0.21.0 (Homebrew)`, `prettier 3.8.3`, `yamlfix 1.19.1`, `panache 2.46.0`, `mdformat 1.0.0`.

The tables use the latest complete checked-in
[single-file results](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/2518d01011b394031bf48f3f8f6b312d48c8deda.json) and
[directory results](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/2518d01011b394031bf48f3f8f6b312d48c8deda.json). The artifacts
record the full per-run timings, output hashes, git commit, and host details. A
table renders only when every tool in its roster completes the benchmark.

## Reproducing

```sh
tools/bench/big.py
tools/bench/run.py --invocation directory --files 500 --items 540 \
  --reps 3 --warmups 1 \
  --tools yamark,yamlfmt,prettier,yamlfix,dprint-yaml,deno-fmt
```

The corpora are generated deterministically, so the same commands reproduce
the same inputs anywhere; each script writes a JSON artifact under
`docs/benchmarks/`.

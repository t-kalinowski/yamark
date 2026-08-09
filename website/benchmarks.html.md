---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 78–115 ms for each generated 4 MB file and 158 ms for 500 generated YAML files (50 MB).

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
   <td style="text-align:right;"> 113 ms </td>
   <td style="text-align:left;"> dprint · 359 ms </td>
   <td style="text-align:right;"> 3.2× </td>
   <td style="text-align:left;"> Both rewrite Markdown </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 4 MB YAML </td>
   <td style="text-align:right;"> 78 ms </td>
   <td style="text-align:left;"> yamlfmt · 190 ms </td>
   <td style="text-align:right;"> 2.4× </td>
   <td style="text-align:left;"> Both rewrite YAML </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 4 MB Markdown + 200 KB YAML front matter </td>
   <td style="text-align:right;"> 115 ms </td>
   <td style="text-align:left;"> dprint · 348 ms </td>
   <td style="text-align:right;"> 3.0× </td>
   <td style="text-align:left;"> dprint leaves YAML front matter untouched </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 500 YAML files (50 MB) </td>
   <td style="text-align:right;"> 158 ms </td>
   <td style="text-align:left;"> Deno · 2.3 s </td>
   <td style="text-align:right;"> 14.3× </td>
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
<div class="benchmark-chart-fallback"><p>Exact values:</p><ul><li><strong>4 MB Markdown - Yamark:</strong> 113 ms</li><li><strong>4 MB Markdown - dprint:</strong> 359 ms</li><li><strong>4 MB Markdown - Panache:</strong> 406 ms</li><li><strong>4 MB Markdown - Deno:</strong> 408 ms</li><li><strong>4 MB Markdown - Prettier:</strong> 1.8 s</li><li><strong>4 MB Markdown - mdformat:</strong> 3.2 s</li><li><strong>4 MB YAML - Yamark:</strong> 78 ms</li><li><strong>4 MB YAML - yamlfmt:</strong> 190 ms</li><li><strong>4 MB YAML - Deno:</strong> 777 ms</li><li><strong>4 MB YAML - dprint:</strong> 1.1 s</li><li><strong>4 MB YAML - yamlfix:</strong> 7.4 s</li><li><strong>4 MB YAML - Prettier:</strong> 20.1 s (file unchanged)</li><li><strong>4 MB Markdown + front matter - Yamark:</strong> 115 ms (formatted)</li><li><strong>4 MB Markdown + front matter - dprint:</strong> 348 ms (untouched)</li><li><strong>4 MB Markdown + front matter - Deno:</strong> 418 ms (formatted)</li><li><strong>4 MB Markdown + front matter - Prettier:</strong> 1.9 s (formatted)</li><li><strong>4 MB Markdown + front matter - mdformat:</strong> 3.8 s (not preserved)</li><li><strong>4 MB Markdown + front matter - Panache:</strong> 6.7 s (formatted)</li><li><strong>500 YAML files · 50 MB - Yamark:</strong> 158 ms</li><li><strong>500 YAML files · 50 MB - Deno:</strong> 2.3 s</li><li><strong>500 YAML files · 50 MB - dprint:</strong> 2.7 s</li><li><strong>500 YAML files · 50 MB - yamlfmt:</strong> 3.9 s</li><li><strong>500 YAML files · 50 MB - Prettier:</strong> 47.4 s</li><li><strong>500 YAML files · 50 MB - yamlfix:</strong> 183.5 s</li></ul></div>
<script type="application/json" id="benchmark-full-field-data">[{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Yamark","seconds":0.1126175619428977,"duration":"113 ms","is_yamark":true,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"dprint","seconds":0.3589440209325403,"duration":"359 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Panache","seconds":0.4059670210117474,"duration":"406 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Deno","seconds":0.4075945625081658,"duration":"408 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Prettier","seconds":1.814105249941349,"duration":"1.8 s","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"mdformat","seconds":3.241150583373383,"duration":"3.2 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Yamark","seconds":0.07822704198770225,"duration":"78 ms","is_yamark":true,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfmt","seconds":0.1900328961201012,"duration":"190 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Deno","seconds":0.7769087500637397,"duration":"777 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"dprint","seconds":1.143036437570117,"duration":"1.1 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfix","seconds":7.376165624940768,"duration":"7.4 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Prettier","seconds":20.144398229429498,"duration":"20.1 s","is_yamark":false,"outcome":"file unchanged"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Yamark","seconds":0.1145400630775839,"duration":"115 ms","is_yamark":true,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"dprint","seconds":0.3479785829549655,"duration":"348 ms","is_yamark":false,"outcome":"untouched"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Deno","seconds":0.418059729039669,"duration":"418 ms","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Prettier","seconds":1.90301912499126,"duration":"1.9 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"mdformat","seconds":3.832500186981633,"duration":"3.8 s","is_yamark":false,"outcome":"not preserved"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Panache","seconds":6.664456958067603,"duration":"6.7 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Yamark","seconds":0.1578086251392961,"duration":"158 ms","is_yamark":true,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Deno","seconds":2.251173458993435,"duration":"2.3 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"dprint","seconds":2.652126084081829,"duration":"2.7 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfmt","seconds":3.923564207972959,"duration":"3.9 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Prettier","seconds":47.350564582971856,"duration":"47.4 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfix","seconds":183.51191100012511,"duration":"183.5 s","is_yamark":false,"outcome":null}]</script>
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
   <td style="text-align:right;"> 113 ms </td>
   <td style="text-align:right;"> 12.5 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 359 ms </td>
   <td style="text-align:right;"> 166.3 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 406 ms </td>
   <td style="text-align:right;"> 31.9 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 408 ms </td>
   <td style="text-align:right;"> 329.4 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.8 s </td>
   <td style="text-align:right;"> 584.9 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.2 s </td>
   <td style="text-align:right;"> 152.9 MB </td>
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
   <td style="text-align:right;"> 51.3 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 190 ms </td>
   <td style="text-align:right;"> 298.2 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 777 ms </td>
   <td style="text-align:right;"> 140.2 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 1.1 s </td>
   <td style="text-align:right;"> 118.3 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 7.4 s </td>
   <td style="text-align:right;"> 264.0 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 20.1 s </td>
   <td style="text-align:right;"> 1061.2 MB </td>
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
   <td style="text-align:right;"> 115 ms </td>
   <td style="text-align:right;"> 15.0 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 348 ms </td>
   <td style="text-align:right;"> 166.5 MB </td>
   <td style="text-align:center;"> untouched </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 418 ms </td>
   <td style="text-align:right;"> 318.7 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.9 s </td>
   <td style="text-align:right;"> 579.7 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.8 s </td>
   <td style="text-align:right;"> 179.2 MB </td>
   <td style="text-align:center;"> not preserved </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 6.7 s </td>
   <td style="text-align:right;"> 39.8 MB </td>
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
   <td style="text-align:right;"> 0.158 s </td>
   <td style="text-align:right;"> 1.245 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 2.251 s </td>
   <td style="text-align:right;"> 32.639 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 2.652 s </td>
   <td style="text-align:right;"> 36.987 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 3.924 s </td>
   <td style="text-align:right;"> 4.318 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 47.351 s </td>
   <td style="text-align:right;"> 73.560 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 183.512 s </td>
   <td style="text-align:right;"> 181.431 s </td>
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
Yamark `0.2.1`, built from commit
`fd88cdd90e20`.
Tool versions: `yamark 0.2.1`, `deno 2.9.4 (stable, release, aarch64-apple-darwin)`, `dprint 0.55.2`, `yamlfmt 0.21.0 (Homebrew)`, `prettier 3.8.3`, `yamlfix 1.19.1`, `panache 2.46.0`, `mdformat 1.0.0`.

The tables use the latest complete checked-in
[single-file results](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/fd88cdd90e20cecab0ca260a7145d18e17c66b6f.json) and
[directory results](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/fd88cdd90e20cecab0ca260a7145d18e17c66b6f.json). The artifacts
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

---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 79–89 ms for each generated 4 MB file and 133 ms for 500 generated YAML files (50 MB).

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
   <td style="text-align:right;"> 89 ms </td>
   <td style="text-align:left;"> dprint · 327 ms </td>
   <td style="text-align:right;"> 3.7× </td>
   <td style="text-align:left;"> Both rewrite Markdown </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 4 MB YAML </td>
   <td style="text-align:right;"> 79 ms </td>
   <td style="text-align:left;"> yamlfmt · 184 ms </td>
   <td style="text-align:right;"> 2.3× </td>
   <td style="text-align:left;"> Both rewrite YAML </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 4 MB Markdown + 200 KB YAML front matter </td>
   <td style="text-align:right;"> 89 ms </td>
   <td style="text-align:left;"> dprint · 317 ms </td>
   <td style="text-align:right;"> 3.6× </td>
   <td style="text-align:left;"> dprint leaves YAML front matter untouched </td>
  </tr>
  <tr>
   <td style="text-align:left;"> 500 YAML files (50 MB) </td>
   <td style="text-align:right;"> 133 ms </td>
   <td style="text-align:left;"> dprint · 2.6 s </td>
   <td style="text-align:right;"> 19.5× </td>
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
<div class="benchmark-chart-fallback"><p>Exact values:</p><ul><li><strong>4 MB Markdown - Yamark:</strong> 89 ms</li><li><strong>4 MB Markdown - dprint:</strong> 327 ms</li><li><strong>4 MB Markdown - Panache:</strong> 398 ms</li><li><strong>4 MB Markdown - Deno:</strong> 407 ms</li><li><strong>4 MB Markdown - Prettier:</strong> 1.5 s</li><li><strong>4 MB Markdown - mdformat:</strong> 2.8 s</li><li><strong>4 MB YAML - Yamark:</strong> 79 ms</li><li><strong>4 MB YAML - yamlfmt:</strong> 184 ms</li><li><strong>4 MB YAML - Deno:</strong> 800 ms</li><li><strong>4 MB YAML - dprint:</strong> 1.0 s</li><li><strong>4 MB YAML - yamlfix:</strong> 7.1 s</li><li><strong>4 MB YAML - Prettier:</strong> 18.7 s (file unchanged)</li><li><strong>4 MB Markdown + front matter - Yamark:</strong> 89 ms (formatted)</li><li><strong>4 MB Markdown + front matter - dprint:</strong> 317 ms (untouched)</li><li><strong>4 MB Markdown + front matter - Deno:</strong> 446 ms (formatted)</li><li><strong>4 MB Markdown + front matter - Prettier:</strong> 1.6 s (formatted)</li><li><strong>4 MB Markdown + front matter - mdformat:</strong> 3.2 s (not preserved)</li><li><strong>4 MB Markdown + front matter - Panache:</strong> 6.8 s (formatted)</li><li><strong>500 YAML files · 50 MB - Yamark:</strong> 133 ms</li><li><strong>500 YAML files · 50 MB - dprint:</strong> 2.6 s</li><li><strong>500 YAML files · 50 MB - Deno:</strong> 3.2 s</li><li><strong>500 YAML files · 50 MB - yamlfmt:</strong> 3.6 s</li><li><strong>500 YAML files · 50 MB - Prettier:</strong> 41.6 s</li><li><strong>500 YAML files · 50 MB - yamlfix:</strong> 175.6 s</li></ul></div>
<script type="application/json" id="benchmark-full-field-data">[{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Yamark","seconds":0.08921762500540353,"duration":"89 ms","is_yamark":true,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"dprint","seconds":0.3271169379877392,"duration":"327 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Panache","seconds":0.3975499170192052,"duration":"398 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Deno","seconds":0.4074254375009332,"duration":"407 ms","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"Prettier","seconds":1.543326353974408,"duration":"1.5 s","is_yamark":false,"outcome":null},{"workload_id":"markdown","short_workload":"4 MB Markdown","workload_order":1,"formatter":"mdformat","seconds":2.827889416483231,"duration":"2.8 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Yamark","seconds":0.07946993751102127,"duration":"79 ms","is_yamark":true,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfmt","seconds":0.1837334164883941,"duration":"184 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Deno","seconds":0.8002548539661802,"duration":"800 ms","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"dprint","seconds":1.033272791508352,"duration":"1.0 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"yamlfix","seconds":7.096864062506938,"duration":"7.1 s","is_yamark":false,"outcome":null},{"workload_id":"yaml","short_workload":"4 MB YAML","workload_order":2,"formatter":"Prettier","seconds":18.672253437485779,"duration":"18.7 s","is_yamark":false,"outcome":"file unchanged"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Yamark","seconds":0.08882020850433037,"duration":"89 ms","is_yamark":true,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"dprint","seconds":0.3168731664773077,"duration":"317 ms","is_yamark":false,"outcome":"untouched"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Deno","seconds":0.4455461875186302,"duration":"446 ms","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Prettier","seconds":1.615801874984754,"duration":"1.6 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"mdformat","seconds":3.224468520493247,"duration":"3.2 s","is_yamark":false,"outcome":"not preserved"},{"workload_id":"frontmatter","short_workload":"4 MB Markdown + front matter","workload_order":3,"formatter":"Panache","seconds":6.759372124972288,"duration":"6.8 s","is_yamark":false,"outcome":"formatted"},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Yamark","seconds":0.1325313340057619,"duration":"133 ms","is_yamark":true,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"dprint","seconds":2.590782792016398,"duration":"2.6 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Deno","seconds":3.233484791009687,"duration":"3.2 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfmt","seconds":3.570427665952593,"duration":"3.6 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"Prettier","seconds":41.60775416699471,"duration":"41.6 s","is_yamark":false,"outcome":null},{"workload_id":"directory","short_workload":"500 YAML files · 50 MB","workload_order":4,"formatter":"yamlfix","seconds":175.64967833296396,"duration":"175.6 s","is_yamark":false,"outcome":null}]</script>
</figure>

## Detailed results

::: {.panel-tabset}

### Markdown

One generated 4 MB Markdown document (`big.md`): prose paragraphs, links, and
nested lists. Each tool's CLI formats a fresh copy. Wall time includes process
startup and is the median of 20 measured runs after 2 warmup runs. Peak
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
   <td style="text-align:right;"> 89 ms </td>
   <td style="text-align:right;"> 20.9 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 327 ms </td>
   <td style="text-align:right;"> 172.6 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 398 ms </td>
   <td style="text-align:right;"> 31.8 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 407 ms </td>
   <td style="text-align:right;"> 478.4 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.5 s </td>
   <td style="text-align:right;"> 522.7 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 2.8 s </td>
   <td style="text-align:right;"> 135.2 MB </td>
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
   <td style="text-align:right;"> 79 ms </td>
   <td style="text-align:right;"> 53.1 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 184 ms </td>
   <td style="text-align:right;"> 295.0 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 800 ms </td>
   <td style="text-align:right;"> 138.1 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 1.0 s </td>
   <td style="text-align:right;"> 118.6 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 7.1 s </td>
   <td style="text-align:right;"> 271.8 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 18.7 s </td>
   <td style="text-align:right;"> 1057.8 MB </td>
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
   <td style="text-align:right;"> 89 ms </td>
   <td style="text-align:right;"> 22.6 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 317 ms </td>
   <td style="text-align:right;"> 172.8 MB </td>
   <td style="text-align:center;"> untouched </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 446 ms </td>
   <td style="text-align:right;"> 583.4 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.6 s </td>
   <td style="text-align:right;"> 546.2 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.2 s </td>
   <td style="text-align:right;"> 132.4 MB </td>
   <td style="text-align:center;"> not preserved </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 6.8 s </td>
   <td style="text-align:right;"> 40.6 MB </td>
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
500 files. Wall time is the median of 7 measured runs after 1 warmup run.

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
   <td style="text-align:right;"> 0.133 s </td>
   <td style="text-align:right;"> 1.219 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 2.591 s </td>
   <td style="text-align:right;"> 37.139 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 3.233 s </td>
   <td style="text-align:right;"> 46.809 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 3.570 s </td>
   <td style="text-align:right;"> 4.011 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 41.608 s </td>
   <td style="text-align:right;"> 61.247 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 175.650 s </td>
   <td style="text-align:right;"> 175.384 s </td>
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

Wall time can vary with system load and scheduling, especially for the parallel
directory workload.

Measured on a MacBook Pro (Apple M4 Max, macOS arm64) using
Yamark `0.3.0+dev`, built from commit
`240432ea9f68` with `cargo build --release --locked`.
Tool versions: `yamark 0.3.0+dev`, `dprint 0.57.4`, `deno 2.9.7 (stable, release, aarch64-apple-darwin)`, `yamlfmt 0.21.0 (Homebrew)`, `prettier 3.8.3`, `yamlfix 1.19.1`, `panache 2.46.0`, `mdformat 1.0.0`.

The tables use the latest complete checked-in
[single-file results](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/240432ea9f6829d4f783514ceba42c3aac0a6e18.json) and
[directory results](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/240432ea9f6829d4f783514ceba42c3aac0a6e18.json). The artifacts
record the full per-run timings, output hashes, git commit, and host details. A
table renders only when every tool in its roster completes the benchmark.

## Reproducing

```sh
tools/bench/big.py --reps 20 --warmups 2
tools/bench/run.py --invocation directory --files 500 --items 540 \
  --reps 7 --warmups 1 \
  --tools yamark,yamlfmt,prettier,yamlfix,dprint-yaml,deno-fmt
```

The corpora are generated deterministically, so the same commands reproduce
the same inputs anywhere; each script writes a JSON artifact under
`docs/benchmarks/`.

Run `tools/bench/run-all.py` for the complete suite using one frozen release
binary. It saves the artifacts, logs, retained corpora, and a manifest under
`target/bench-all/`.

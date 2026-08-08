---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**On the benchmark host (Apple M4 Max, macOS arm64), Yamark recorded the lowest elapsed time in all four workloads:** 82–114 ms for each generated 4 MB file and 188 ms for 500 generated YAML files (50 MB).

Each table compares the formatter CLIs in this harness that accept that input.
Tools use their default formatting behavior, with no formatting options, shims,
or adapters, so the roster differs by input kind.

::: {.panel-tabset}

## Markdown

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
   <td style="text-align:right;"> 114 ms </td>
   <td style="text-align:right;"> 12.8 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 358 ms </td>
   <td style="text-align:right;"> 166.4 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 411 ms </td>
   <td style="text-align:right;"> 329.4 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 413 ms </td>
   <td style="text-align:right;"> 31.9 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.8 s </td>
   <td style="text-align:right;"> 581.7 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.3 s </td>
   <td style="text-align:right;"> 153.0 MB </td>
  </tr>
</tbody>
</table>

## YAML

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
   <td style="text-align:right;"> 82 ms </td>
   <td style="text-align:right;"> 51.5 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 194 ms </td>
   <td style="text-align:right;"> 263.9 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 787 ms </td>
   <td style="text-align:right;"> 140.1 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 1.2 s </td>
   <td style="text-align:right;"> 118.3 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 7.5 s </td>
   <td style="text-align:right;"> 264.0 MB </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 20.3 s </td>
   <td style="text-align:right;"> 1061.5 MB </td>
  </tr>
</tbody>
</table>

## Markdown + front matter

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
   <td style="text-align:right;"> 112 ms </td>
   <td style="text-align:right;"> 15.3 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 346 ms </td>
   <td style="text-align:right;"> 166.5 MB </td>
   <td style="text-align:center;"> untouched </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 457 ms </td>
   <td style="text-align:right;"> 318.7 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.9 s </td>
   <td style="text-align:right;"> 573.7 MB </td>
   <td style="text-align:center;"> formatted </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.8 s </td>
   <td style="text-align:right;"> 179.3 MB </td>
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

## Directory

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
   <td style="text-align:right;"> 0.188 s </td>
   <td style="text-align:right;"> 1.176 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 2.743 s </td>
   <td style="text-align:right;"> 34.080 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 3.386 s </td>
   <td style="text-align:right;"> 38.306 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 4.100 s </td>
   <td style="text-align:right;"> 4.519 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 49.826 s </td>
   <td style="text-align:right;"> 75.237 s </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 188.521 s </td>
   <td style="text-align:right;"> 185.090 s </td>
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

Measured on a MacBook Pro (Apple M4 Max, macOS arm64) with Yamark built in Cargo's
release profile. Tool versions: `yamark 0.2.0`, `deno 2.9.4 (stable, release, aarch64-apple-darwin)`, `dprint 0.55.2`, `yamlfmt 0.21.0 (Homebrew)`, `prettier 3.8.3`, `yamlfix 1.19.1`, `panache 2.46.0`, `mdformat 1.0.0`.

The tables use the latest complete checked-in
[single-file results](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/637698dd7ec135ae8242038875fab930efe4c4aa.json) and
[directory results](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/637698dd7ec135ae8242038875fab930efe4c4aa.json). The artifacts
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

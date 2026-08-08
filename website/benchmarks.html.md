---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**Yamark formats a 4 MB Markdown document in 114 ms and a 4 MB YAML file in 82 ms.** The next-fastest tool on each is `dprint-markdown` (358 ms) and `yamlfmt` (194 ms). On a directory of 500 YAML files (50 MB), Yamark finishes in 188 ms; the next-fastest formatter, `deno-fmt`, takes 2.7 s.

There is one comparison per input kind. Each table includes the tools in this
harness whose own CLI accepts that input, with no formatting options, shims, or
adapters. The tool roster therefore differs by input kind.

::: {.panel-tabset}

## Markdown

One generated 4 MB Markdown document (`big.md`): prose paragraphs with
links - some longer than the line width - and nested lists with mixed
markers. Each tool's CLI formats the file in place; time includes process
startup. Time is the median of 10 measured runs after 2 warmup runs,
on a fresh copy of the file each run. Memory is median peak RSS.

<table class="perf-table">
 <thead>
  <tr>
   <th style="text-align:left;"> Formatter </th>
   <th style="text-align:right;"> Time </th>
   <th style="text-align:right;"> Memory </th>
   <th style="text-align:right;"> vs yamark </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> yamark </td>
   <td style="text-align:right;"> 114 ms </td>
   <td style="text-align:right;"> 12.8 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 358 ms </td>
   <td style="text-align:right;"> 166.4 MB </td>
   <td style="text-align:right;"> 3.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 411 ms </td>
   <td style="text-align:right;"> 329.4 MB </td>
   <td style="text-align:right;"> 3.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 413 ms </td>
   <td style="text-align:right;"> 31.9 MB </td>
   <td style="text-align:right;"> 3.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.8 s </td>
   <td style="text-align:right;"> 581.7 MB </td>
   <td style="text-align:right;"> 16.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.3 s </td>
   <td style="text-align:right;"> 153.0 MB </td>
   <td style="text-align:right;"> 28.8x slower </td>
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
   <th style="text-align:right;"> Time </th>
   <th style="text-align:right;"> Memory </th>
   <th style="text-align:right;"> vs yamark </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> yamark </td>
   <td style="text-align:right;"> 82 ms </td>
   <td style="text-align:right;"> 51.5 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 194 ms </td>
   <td style="text-align:right;"> 263.9 MB </td>
   <td style="text-align:right;"> 2.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 787 ms </td>
   <td style="text-align:right;"> 140.1 MB </td>
   <td style="text-align:right;"> 9.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 1.2 s </td>
   <td style="text-align:right;"> 118.3 MB </td>
   <td style="text-align:right;"> 14.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 7.5 s </td>
   <td style="text-align:right;"> 264.0 MB </td>
   <td style="text-align:right;"> 92.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 20.3 s </td>
   <td style="text-align:right;"> 1061.5 MB </td>
   <td style="text-align:right;"> 247.9x slower </td>
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
   <th style="text-align:right;"> Time </th>
   <th style="text-align:right;"> Memory </th>
   <th style="text-align:center;"> Front matter </th>
   <th style="text-align:right;"> vs yamark </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> yamark </td>
   <td style="text-align:right;"> 112 ms </td>
   <td style="text-align:right;"> 15.3 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 346 ms </td>
   <td style="text-align:right;"> 166.5 MB </td>
   <td style="text-align:center;"> untouched </td>
   <td style="text-align:right;"> 3.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 457 ms </td>
   <td style="text-align:right;"> 318.7 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 4.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.9 s </td>
   <td style="text-align:right;"> 573.7 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 16.9x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.8 s </td>
   <td style="text-align:right;"> 179.3 MB </td>
   <td style="text-align:center;"> not preserved </td>
   <td style="text-align:right;"> 34.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 6.7 s </td>
   <td style="text-align:right;"> 39.8 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 59.9x slower </td>
  </tr>
</tbody>
</table>

The harness derives that column from the output bytes: trailing-whitespace
trimming does not count as formatting. `dprint`'s Markdown plugin passes
front matter through unformatted. `mdformat`, installed without its front-matter
plugin, reads the opening `---` as a thematic break and does not preserve the
front matter delimiters.

## Directory

500 generated YAML service-configuration
files of about 100 KB each
(50 MB in total). Each
tool is passed the directory root once and discovers the files itself;
every run is verified to have reformatted all
500 files. Time is
the median of 3 measured runs after 1 warmup run, on a fresh copy of the corpus each run.

<table class="perf-table">
 <thead>
  <tr>
   <th style="text-align:left;"> Formatter </th>
   <th style="text-align:right;"> Time </th>
   <th style="text-align:right;"> User CPU </th>
   <th style="text-align:right;"> Throughput </th>
   <th style="text-align:right;"> vs yamark </th>
  </tr>
 </thead>
<tbody>
  <tr>
   <td style="text-align:left;"> yamark </td>
   <td style="text-align:right;"> 0.188 s </td>
   <td style="text-align:right;"> 1.176 s </td>
   <td style="text-align:right;"> 265.4 MB/s </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 2.743 s </td>
   <td style="text-align:right;"> 34.080 s </td>
   <td style="text-align:right;"> 18.2 MB/s </td>
   <td style="text-align:right;"> 14.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 3.386 s </td>
   <td style="text-align:right;"> 38.306 s </td>
   <td style="text-align:right;"> 14.8 MB/s </td>
   <td style="text-align:right;"> 18.0x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 4.100 s </td>
   <td style="text-align:right;"> 4.519 s </td>
   <td style="text-align:right;"> 12.2 MB/s </td>
   <td style="text-align:right;"> 21.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 49.826 s </td>
   <td style="text-align:right;"> 75.237 s </td>
   <td style="text-align:right;"> 1.00 MB/s </td>
   <td style="text-align:right;"> 264.7x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 188.521 s </td>
   <td style="text-align:right;"> 185.090 s </td>
   <td style="text-align:right;"> 0.27 MB/s </td>
   <td style="text-align:right;"> 1001.4x slower </td>
  </tr>
</tbody>
</table>

Yamark formats the directory in parallel (as do `deno-fmt` and `dprint`),
so wall time can beat user CPU time; the User CPU column is the
single-core comparison, and Yamark is fastest on that column too.
Throughput is input MB/s.

:::

## How to read these results

Every number comes from the same harness (`tools/bench/big.py` and
`tools/bench/run.py` in the repository). It runs each included tool through its
own CLI with default configuration and no formatting options against
deterministic generated corpora. The current tool set is
`deno-fmt`, `dprint-markdown`, `dprint-yaml`, `mdformat`, `panache`, `prettier`, `yamlfix`, `yamlfmt`.

Two harness details prevent cached work or plugin defaults from affecting the
comparison without changing how any tool is invoked:

- `panache` and `dprint` cache format results. The harness redirects each
  tool's cache into the benchmark work directory and clears it between
  repetitions, so every timed run formats from scratch rather than
  replaying a cached result.
- `dprint` has no built-in plugins, so it runs with a config file that
  names its first-party plugin for the input kind (Markdown or YAML) and
  sets nothing else.

Lint fixers with an autofix mode (`pymarkdown`, `markdownlint-cli2`) are outside
this formatter-CLI comparison. The harness can still run them via `--tools`.

Measured on a MacBook Pro (Apple M4 Max, macOS arm64). Tool versions: `yamark 0.2.0`, `deno 2.9.4 (stable, release, aarch64-apple-darwin)`, `dprint 0.55.2`, `yamlfmt 0.21.0 (Homebrew)`, `prettier 3.8.3`, `yamlfix 1.19.1`, `panache 2.46.0`, `mdformat 1.0.0`.

The tables render the latest checked-in artifacts -
[`637698d`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/637698dd7ec135ae8242038875fab930efe4c4aa.json) for the single-file comparisons and
[`637698d`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/637698dd7ec135ae8242038875fab930efe4c4aa.json) for the directory comparison - which
record the full per-run timings, output hashes, git commit, and host
details. A table renders only if every tool in its roster completed the
benchmark; degraded runs are never shown as smaller tables.

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

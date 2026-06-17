---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**Yamark formats a 4 MB Markdown document in 115 ms and a 4 MB YAML file in 83 ms.** The next-fastest tool on each is `dprint-markdown` (338 ms) and `yamlfmt` (177 ms). On a directory of 500 YAML files (50 MB), yamark finishes in 123 ms; the next-fastest formatter, `deno-fmt`, takes 2.1 s.

There is one comparison per input kind, and each lists every tool whose own
CLI formats that input natively, used simply: no formatting options, no
shims, no adapters. The tool roster therefore differs by input kind.

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
   <td style="text-align:right;"> 115 ms </td>
   <td style="text-align:right;"> 12.7 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 338 ms </td>
   <td style="text-align:right;"> 177.3 MB </td>
   <td style="text-align:right;"> 2.9x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 367 ms </td>
   <td style="text-align:right;"> 482.2 MB </td>
   <td style="text-align:right;"> 3.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 408 ms </td>
   <td style="text-align:right;"> 32.5 MB </td>
   <td style="text-align:right;"> 3.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.8 s </td>
   <td style="text-align:right;"> 561.9 MB </td>
   <td style="text-align:right;"> 15.5x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.3 s </td>
   <td style="text-align:right;"> 127.9 MB </td>
   <td style="text-align:right;"> 28.6x slower </td>
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
   <td style="text-align:right;"> 83 ms </td>
   <td style="text-align:right;"> 51.2 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 177 ms </td>
   <td style="text-align:right;"> 292.0 MB </td>
   <td style="text-align:right;"> 2.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 655 ms </td>
   <td style="text-align:right;"> 143.6 MB </td>
   <td style="text-align:right;"> 7.9x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 812 ms </td>
   <td style="text-align:right;"> 120.2 MB </td>
   <td style="text-align:right;"> 9.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 8.0 s </td>
   <td style="text-align:right;"> 275.0 MB </td>
   <td style="text-align:right;"> 96.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 19.7 s </td>
   <td style="text-align:right;"> 1100.1 MB </td>
   <td style="text-align:right;"> 236.7x slower </td>
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
   <td style="text-align:right;"> 109 ms </td>
   <td style="text-align:right;"> 15.2 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 330 ms </td>
   <td style="text-align:right;"> 177.5 MB </td>
   <td style="text-align:center;"> untouched </td>
   <td style="text-align:right;"> 3.0x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 421 ms </td>
   <td style="text-align:right;"> 587.7 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 3.9x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.9 s </td>
   <td style="text-align:right;"> 570.0 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 17.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.9 s </td>
   <td style="text-align:right;"> 132.5 MB </td>
   <td style="text-align:center;"> not preserved </td>
   <td style="text-align:right;"> 35.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 6.5 s </td>
   <td style="text-align:right;"> 38.6 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 59.5x slower </td>
  </tr>
</tbody>
</table>

The harness derives that column from the output bytes: trailing-whitespace
trimming does not count as formatting. `dprint`'s Markdown plugin passes
front matter through unformatted, and `mdformat` (installed without its
front-matter plugin) reads the opening `---` as a thematic break and
corrupts the block.

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
   <td style="text-align:right;"> 0.123 s </td>
   <td style="text-align:right;"> 1.146 s </td>
   <td style="text-align:right;"> 407.3 MB/s </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 2.124 s </td>
   <td style="text-align:right;"> 32.005 s </td>
   <td style="text-align:right;"> 23.5 MB/s </td>
   <td style="text-align:right;"> 17.3x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 2.548 s </td>
   <td style="text-align:right;"> 37.125 s </td>
   <td style="text-align:right;"> 19.6 MB/s </td>
   <td style="text-align:right;"> 20.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 3.687 s </td>
   <td style="text-align:right;"> 4.089 s </td>
   <td style="text-align:right;"> 13.6 MB/s </td>
   <td style="text-align:right;"> 30.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 42.838 s </td>
   <td style="text-align:right;"> 61.571 s </td>
   <td style="text-align:right;"> 1.17 MB/s </td>
   <td style="text-align:right;"> 349.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 194.987 s </td>
   <td style="text-align:right;"> 192.535 s </td>
   <td style="text-align:right;"> 0.26 MB/s </td>
   <td style="text-align:right;"> 1589.2x slower </td>
  </tr>
</tbody>
</table>

Yamark formats the directory in parallel (as do `deno-fmt` and `dprint`),
so wall time can beat user CPU time; the User CPU column is the
single-core comparison, and yamark is fastest on that column too.
Throughput is input MB/s.

:::

## How to read these results

Every number comes from the same harness (`tools/bench/big.py` and
`tools/bench/run.py` in the repository), running each tool the way a user
would: its own CLI, default configuration, no formatting options, against
deterministic generated corpora. The comparison set is the other formatters
in the space - `deno-fmt`, `dprint-markdown`, `dprint-yaml`, `mdformat`, `panache`, `prettier`, `yamlfix`, `yamlfmt`.

Two harness details keep that comparison clean without changing how any
tool is invoked:

- `panache` and `dprint` cache format results. The harness redirects each
  tool's cache into the benchmark work directory and clears it between
  repetitions, so every timed run formats from scratch rather than
  replaying a cached result.
- `dprint` has no built-in plugins, so it runs with a config file that
  names its first-party plugin for the input kind (Markdown or YAML) and
  sets nothing else.

Lint fixers with an autofix mode (`pymarkdown`, `markdownlint-cli2`) are
not formatters, so they are not part of the comparison. The harness can
still run them via `--tools`.

Measured on a MacBook Pro (Apple M4 Max, macOS arm64). Tool versions: `yamark 0.1.0`, `deno 2.8.3 (stable, release, aarch64-apple-darwin)`, `dprint 0.54.0`, `yamlfmt 0.21.0 (Homebrew)`, `prettier 3.8.3`, `yamlfix 1.19.1`, `panache 2.46.0`, `mdformat 1.0.0`.

The tables render the latest checked-in artifacts -
[`dfb7e73`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/dfb7e738fb58fc2ac9ffcae4bdac8829ed2d32d3.json) for the single-file comparisons and
[`6932f25`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/6932f25d4f83db5c3a04307540424d5e7c74f2a2.json) for the directory comparison - which
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

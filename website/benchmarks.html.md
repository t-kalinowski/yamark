---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**Yamark formats a 4 MB Markdown document in 106 ms and a 4 MB YAML file in 75 ms.** The next-fastest tool on each is `dprint-markdown` (326 ms) and `yamlfmt` (193 ms). On a directory of 500 YAML files (50 MB), yamark finishes in 137 ms; the next-fastest formatter, `deno-fmt`, takes 2.6 s.

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
   <td style="text-align:right;"> 106 ms </td>
   <td style="text-align:right;"> 12.6 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 326 ms </td>
   <td style="text-align:right;"> 172.8 MB </td>
   <td style="text-align:right;"> 3.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 360 ms </td>
   <td style="text-align:right;"> 481.6 MB </td>
   <td style="text-align:right;"> 3.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 387 ms </td>
   <td style="text-align:right;"> 32.1 MB </td>
   <td style="text-align:right;"> 3.7x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.6 s </td>
   <td style="text-align:right;"> 582.3 MB </td>
   <td style="text-align:right;"> 15.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 2.9 s </td>
   <td style="text-align:right;"> 152.9 MB </td>
   <td style="text-align:right;"> 27.6x slower </td>
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
   <td style="text-align:right;"> 75 ms </td>
   <td style="text-align:right;"> 51.6 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 193 ms </td>
   <td style="text-align:right;"> 283.8 MB </td>
   <td style="text-align:right;"> 2.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 761 ms </td>
   <td style="text-align:right;"> 141.7 MB </td>
   <td style="text-align:right;"> 10.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 1.2 s </td>
   <td style="text-align:right;"> 118.7 MB </td>
   <td style="text-align:right;"> 15.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 7.4 s </td>
   <td style="text-align:right;"> 269.5 MB </td>
   <td style="text-align:right;"> 98.3x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 20.1 s </td>
   <td style="text-align:right;"> 845.5 MB </td>
   <td style="text-align:right;"> 267.2x slower </td>
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
   <td style="text-align:right;"> 15.1 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 347 ms </td>
   <td style="text-align:right;"> 172.9 MB </td>
   <td style="text-align:center;"> untouched </td>
   <td style="text-align:right;"> 3.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 415 ms </td>
   <td style="text-align:right;"> 586.3 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 3.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.8 s </td>
   <td style="text-align:right;"> 579.4 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 17.0x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.8 s </td>
   <td style="text-align:right;"> 179.9 MB </td>
   <td style="text-align:center;"> not preserved </td>
   <td style="text-align:right;"> 34.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 6.5 s </td>
   <td style="text-align:right;"> 40.8 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 59.9x slower </td>
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
   <td style="text-align:right;"> 0.137 s </td>
   <td style="text-align:right;"> 1.150 s </td>
   <td style="text-align:right;"> 365.7 MB/s </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 2.596 s </td>
   <td style="text-align:right;"> 33.016 s </td>
   <td style="text-align:right;"> 19.3 MB/s </td>
   <td style="text-align:right;"> 19.0x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 2.865 s </td>
   <td style="text-align:right;"> 38.764 s </td>
   <td style="text-align:right;"> 17.4 MB/s </td>
   <td style="text-align:right;"> 21.0x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 3.978 s </td>
   <td style="text-align:right;"> 4.401 s </td>
   <td style="text-align:right;"> 12.6 MB/s </td>
   <td style="text-align:right;"> 29.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 47.320 s </td>
   <td style="text-align:right;"> 72.922 s </td>
   <td style="text-align:right;"> 1.06 MB/s </td>
   <td style="text-align:right;"> 346.3x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 184.442 s </td>
   <td style="text-align:right;"> 181.767 s </td>
   <td style="text-align:right;"> 0.27 MB/s </td>
   <td style="text-align:right;"> 1349.9x slower </td>
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

Measured on a MacBook Pro (Apple M4 Max, macOS arm64). Tool versions: `yamark 0.1.0`, `deno 2.9.4 (stable, release, aarch64-apple-darwin)`, `dprint 0.55.2`, `yamlfmt 0.21.0 (Homebrew)`, `prettier 3.8.3`, `yamlfix 1.19.1`, `panache 2.46.0`, `mdformat 1.0.0`.

The tables render the latest checked-in artifacts -
[`f49e731`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/f49e7312990e2919990146c35f99e2770bef1b75.json) for the single-file comparisons and
[`f49e731`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/f49e7312990e2919990146c35f99e2770bef1b75.json) for the directory comparison - which
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

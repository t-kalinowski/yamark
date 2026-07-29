---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**Yamark formats a 4 MB Markdown document in 118 ms and a 4 MB YAML file in 77 ms.** The next-fastest tool on each is `dprint-markdown` (346 ms) and `yamlfmt` (189 ms). On a directory of 500 YAML files (50 MB), yamark finishes in 122 ms; the next-fastest formatter, `deno-fmt`, takes 2.6 s.

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
   <td style="text-align:right;"> 118 ms </td>
   <td style="text-align:right;"> 12.8 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 346 ms </td>
   <td style="text-align:right;"> 172.8 MB </td>
   <td style="text-align:right;"> 2.9x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 383 ms </td>
   <td style="text-align:right;"> 481.5 MB </td>
   <td style="text-align:right;"> 3.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 404 ms </td>
   <td style="text-align:right;"> 32.3 MB </td>
   <td style="text-align:right;"> 3.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.7 s </td>
   <td style="text-align:right;"> 582.7 MB </td>
   <td style="text-align:right;"> 14.3x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.1 s </td>
   <td style="text-align:right;"> 152.9 MB </td>
   <td style="text-align:right;"> 26.4x slower </td>
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
   <td style="text-align:right;"> 77 ms </td>
   <td style="text-align:right;"> 51.6 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 189 ms </td>
   <td style="text-align:right;"> 269.0 MB </td>
   <td style="text-align:right;"> 2.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 759 ms </td>
   <td style="text-align:right;"> 142.2 MB </td>
   <td style="text-align:right;"> 9.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 1.2 s </td>
   <td style="text-align:right;"> 118.7 MB </td>
   <td style="text-align:right;"> 15.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 7.5 s </td>
   <td style="text-align:right;"> 269.5 MB </td>
   <td style="text-align:right;"> 96.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 20.5 s </td>
   <td style="text-align:right;"> 1066.4 MB </td>
   <td style="text-align:right;"> 265.6x slower </td>
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
   <td style="text-align:right;"> 121 ms </td>
   <td style="text-align:right;"> 15.3 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 347 ms </td>
   <td style="text-align:right;"> 172.9 MB </td>
   <td style="text-align:center;"> untouched </td>
   <td style="text-align:right;"> 2.9x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 422 ms </td>
   <td style="text-align:right;"> 585.7 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 3.5x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.9 s </td>
   <td style="text-align:right;"> 580.7 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 15.3x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.8 s </td>
   <td style="text-align:right;"> 180.5 MB </td>
   <td style="text-align:center;"> not preserved </td>
   <td style="text-align:right;"> 31.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 6.6 s </td>
   <td style="text-align:right;"> 40.8 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 54.3x slower </td>
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
   <td style="text-align:right;"> 0.122 s </td>
   <td style="text-align:right;"> 1.137 s </td>
   <td style="text-align:right;"> 409.6 MB/s </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 2.589 s </td>
   <td style="text-align:right;"> 33.620 s </td>
   <td style="text-align:right;"> 19.3 MB/s </td>
   <td style="text-align:right;"> 21.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 3.019 s </td>
   <td style="text-align:right;"> 39.495 s </td>
   <td style="text-align:right;"> 16.6 MB/s </td>
   <td style="text-align:right;"> 24.7x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 3.898 s </td>
   <td style="text-align:right;"> 4.302 s </td>
   <td style="text-align:right;"> 12.8 MB/s </td>
   <td style="text-align:right;"> 31.9x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 45.623 s </td>
   <td style="text-align:right;"> 66.232 s </td>
   <td style="text-align:right;"> 1.10 MB/s </td>
   <td style="text-align:right;"> 373.9x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 183.685 s </td>
   <td style="text-align:right;"> 180.549 s </td>
   <td style="text-align:right;"> 0.27 MB/s </td>
   <td style="text-align:right;"> 1505.5x slower </td>
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
[`3c3a4bc`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/3c3a4bccabc9501f5be83060b957e7cfb8cb67af.json) for the single-file comparisons and
[`3c3a4bc`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/3c3a4bccabc9501f5be83060b957e7cfb8cb67af.json) for the directory comparison - which
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

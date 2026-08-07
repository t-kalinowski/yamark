---
title: Benchmarks
description: Yamark performance against other YAML and Markdown formatters.
---



**Yamark formats a 4 MB Markdown document in 109 ms and a 4 MB YAML file in 69 ms.** The next-fastest tool on each is `dprint-markdown` (349 ms) and `yamlfmt` (187 ms). On a directory of 500 YAML files (50 MB), Yamark finishes in 133 ms; the next-fastest formatter, `deno-fmt`, takes 2.6 s.

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
   <td style="text-align:right;"> 109 ms </td>
   <td style="text-align:right;"> 12.8 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 349 ms </td>
   <td style="text-align:right;"> 172.8 MB </td>
   <td style="text-align:right;"> 3.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 376 ms </td>
   <td style="text-align:right;"> 482.0 MB </td>
   <td style="text-align:right;"> 3.5x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 399 ms </td>
   <td style="text-align:right;"> 33.0 MB </td>
   <td style="text-align:right;"> 3.7x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.8 s </td>
   <td style="text-align:right;"> 596.4 MB </td>
   <td style="text-align:right;"> 16.2x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.1 s </td>
   <td style="text-align:right;"> 153.4 MB </td>
   <td style="text-align:right;"> 28.9x slower </td>
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
   <td style="text-align:right;"> 69 ms </td>
   <td style="text-align:right;"> 51.7 MB </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 187 ms </td>
   <td style="text-align:right;"> 248.3 MB </td>
   <td style="text-align:right;"> 2.7x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 761 ms </td>
   <td style="text-align:right;"> 142.9 MB </td>
   <td style="text-align:right;"> 11.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 1.2 s </td>
   <td style="text-align:right;"> 118.7 MB </td>
   <td style="text-align:right;"> 16.7x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 7.3 s </td>
   <td style="text-align:right;"> 268.2 MB </td>
   <td style="text-align:right;"> 106.6x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 20.0 s </td>
   <td style="text-align:right;"> 846.1 MB </td>
   <td style="text-align:right;"> 290.5x slower </td>
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
   <td style="text-align:right;"> 113 ms </td>
   <td style="text-align:right;"> 15.4 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-markdown </td>
   <td style="text-align:right;"> 346 ms </td>
   <td style="text-align:right;"> 173.1 MB </td>
   <td style="text-align:center;"> untouched </td>
   <td style="text-align:right;"> 3.1x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 430 ms </td>
   <td style="text-align:right;"> 587.6 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 3.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 1.9 s </td>
   <td style="text-align:right;"> 580.8 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 16.5x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> mdformat </td>
   <td style="text-align:right;"> 3.8 s </td>
   <td style="text-align:right;"> 180.0 MB </td>
   <td style="text-align:center;"> not preserved </td>
   <td style="text-align:right;"> 33.4x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> panache </td>
   <td style="text-align:right;"> 6.6 s </td>
   <td style="text-align:right;"> 41.0 MB </td>
   <td style="text-align:center;"> formatted </td>
   <td style="text-align:right;"> 58.7x slower </td>
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
   <td style="text-align:right;"> 0.133 s </td>
   <td style="text-align:right;"> 1.087 s </td>
   <td style="text-align:right;"> 374.9 MB/s </td>
   <td style="text-align:right;"> 1x </td>
  </tr>
  <tr>
   <td style="text-align:left;"> deno-fmt </td>
   <td style="text-align:right;"> 2.639 s </td>
   <td style="text-align:right;"> 33.668 s </td>
   <td style="text-align:right;"> 18.9 MB/s </td>
   <td style="text-align:right;"> 19.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> dprint-yaml </td>
   <td style="text-align:right;"> 3.112 s </td>
   <td style="text-align:right;"> 39.949 s </td>
   <td style="text-align:right;"> 16.1 MB/s </td>
   <td style="text-align:right;"> 23.3x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfmt </td>
   <td style="text-align:right;"> 3.967 s </td>
   <td style="text-align:right;"> 4.447 s </td>
   <td style="text-align:right;"> 12.6 MB/s </td>
   <td style="text-align:right;"> 29.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> prettier </td>
   <td style="text-align:right;"> 46.748 s </td>
   <td style="text-align:right;"> 71.863 s </td>
   <td style="text-align:right;"> 1.07 MB/s </td>
   <td style="text-align:right;"> 350.8x slower </td>
  </tr>
  <tr>
   <td style="text-align:left;"> yamlfix </td>
   <td style="text-align:right;"> 179.255 s </td>
   <td style="text-align:right;"> 178.717 s </td>
   <td style="text-align:right;"> 0.28 MB/s </td>
   <td style="text-align:right;"> 1345.0x slower </td>
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

Measured on a MacBook Pro (Apple M4 Max, macOS arm64). Tool versions: `yamark 0.1.0`, `deno 2.9.4 (stable, release, aarch64-apple-darwin)`, `dprint 0.55.2`, `yamlfmt 0.21.0 (Homebrew)`, `prettier 3.8.3`, `yamlfix 1.19.1`, `panache 2.46.0`, `mdformat 1.0.0`.

The tables render the latest checked-in artifacts -
[`ab72bb5`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/big/ab72bb5022506bc5a36a1352c4462102a84865a4.json) for the single-file comparisons and
[`d9bbab3`](https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/yaml/d9bbab37fd9f3bb453e8b53cbe746cb1262802c7.json) for the directory comparison - which
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

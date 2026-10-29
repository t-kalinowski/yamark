# YAML formatter benchmark

This benchmark measures dirty YAML formatting throughput over one deterministic generated corpus. Each timed repetition copies the corpus, then invokes the formatter according to `--invocation`.

## Default run

When asked to "run benchmarks", use the default benchmark path:

```sh
tools/bench/run.py
```

The default path runs only `yamark` in `per-file` mode on the `representative` corpus. It retains the 400-file budget and invokes the formatter separately for each file, avoiding directory-mode internal parallelism. The default remains sized to finish in under two minutes on the usual development machine.

The corpus rotates through eight workload families, with 50 files per family at the default size:

| Family                | Coverage                                                                               |
| --------------------- | -------------------------------------------------------------------------------------- |
| `ordinary-config`     | Short typed values, nested mappings, flow lists, comments, anchors, and aliases.       |
| `flow-heavy`          | The existing dense flow-collection configuration fixture.                              |
| `block-heavy`         | The existing block-collection configuration fixture.                                   |
| `plain-prose`         | Multiline prose with varied word counts and continuation positions.                    |
| `quoted-prose`        | Single and double quotes, Unicode, escaped quotes, and backslashes.                    |
| `literal-scalars`     | Literal blocks, blank lines, and CRLF line endings.                                    |
| `markdown-paragraphs` | Tagged scalars with one, two, and eight paragraphs, with and without sentence periods. |
| `markdown-rich`       | Tagged Markdown with lists, fences, links, inline code, templates, and block quotes.   |

These deterministic synthetic inputs cover selected configuration and scalar shapes; they are not a sample of production YAML usage. New prose uses lorem ipsum text. No external repositories or downloads are needed. Every file starts dirty so the harness continues to reject timed no-op runs.

The default `--items 80` controls entries per file. For Markdown it is a paragraph or block budget, so adding richer structure does not multiply the work by eight. Use at least eight files to include every family. Individual families are also selectable without changing the runner or repetition count:

```sh
tools/bench/run.py --corpus-shape ordinary-config
tools/bench/run.py --corpus-shape markdown-paragraphs --files 8 --items 80
```

The `!md` workloads ask Yamark to format nested Markdown. Other YAML formatters may preserve those values or handle the tag differently; their timings do not necessarily represent the same formatting work.

Then present the recent `yamark` per-file comparison:

```sh
tools/bench/summarize.R
```

By default, the summary shows the last five matching commits for `yamark`, `per-file`, `write`, the `default` width profile, and the `representative` corpus. Historical flow-heavy results remain separate:

```sh
tools/bench/summarize.R --corpus-shape flow-heavy
```

## Compact output

Use the bounded summary path when a compact report is more useful than the full benchmark log:

```sh
tools/bench/run.py --agent-summary
```

This runs the same default benchmark, writes the commit-scoped JSON artifact, and prints the current row, previous matching row, delta, artifact path, and log path.

## Complete suite

Use the complete runner when every canonical benchmark is required:

```sh
tools/bench/run-all.py
```

This long-running command requires a clean worktree. It runs the allocation and benchmark-harness tests, builds one locked release binary, then uses that frozen binary for the representative per-file YAML benchmark, the three-file large-file benchmark, and the published directory comparison. The workload sizes, formatter rosters, repetitions, and warmups are explicit in the runner so a child script's changed defaults cannot silently change the suite. It uses 5 measured repetitions for per-file YAML, 20 for each large file, and 7 for directory YAML, after 1, 2, and 1 warmup repetitions respectively. More repetitions reduce the effect of occasional delays; run on an idle machine to limit sustained host interference.

By default, logs, scratch data, the frozen binary, and three separate JSON artifacts are written under a new `target/bench-all/<timestamp>-<commit>/` directory. A `manifest.json` is written only after every command and artifact validation succeeds. It records hashes for the binary, artifacts, and retained input corpora, as well as the R package versions used to generate the large files. To choose another new directory or inspect the commands without running or writing anything, use:

```sh
tools/bench/run-all.py --result-dir /path/to/results
tools/bench/run-all.py --dry-run --result-dir /path/to/results
```

The full comparison requires Python, `Rscript` with the `stringi` and `yaml12` packages, plus `panache`, `mdformat`, `prettier`, `dprint`, `deno`, `yamlfmt`, and `yamlfix` on `PATH`. Missing comparison tools fail the complete run instead of producing a partial result. The runner does not copy results into `docs/benchmarks`; publishing remains a separate, reviewed step.

Unix measurements use `resource`/`os.wait4`. Windows runs report elapsed time, throughput, and output checks; process-tree CPU and peak RSS measurements are unavailable, recorded as `null` samples and medians with an explanatory `resource_metrics` status. Launcher-only counters would miss work performed by Node or other child processes. Run the Python scripts with `python` on Windows; executable paths include the `.exe` suffix automatically and npm `.cmd` shims are resolved from `PATH`. Tests that install fake Unix shell executables run only on Unix.

## Big single-file benchmark

The big-file benchmark generates three deterministic single-file corpora - `big.md`, `big.yaml`, and `big-with-frontmatter.md`, 4 MB each (the largest round size every comparison tool accepts: panache refuses inputs over 4 MiB) - and times each formatter's own CLI on each file it natively supports:

```sh
tools/bench/big.py
```

The default tool set is the native formatter CLIs, each invoked with no formatting options (`prettier --write FILE`, `panache format FILE`, and so on). Tools that cache format results (`panache`, `dprint`) run with their cache redirected into the benchmark work directory and wiped between repetitions, so every timed run formats from scratch. Lint fixers (`pymarkdown`, `markdownlint-cli2`) and library shims (`panache-yaml`, `pretty-yaml`, `py-yaml12`) are outside the default formatter-CLI comparison; select them explicitly with `--tools` when needed.

The front matter file embeds a deliberately unformatted YAML block sized from the document (5% of bytes, about a third of the lines); override it with `--frontmatter-yaml-bytes`. For each Markdown file with front matter, the artifact records a `front_matter` outcome - `rewritten`, `preserved`, or `removed` - derived from the output bytes. Override all three sizes for a quick smoke run:

```sh
tools/bench/big.py --target-bytes 12000 --tools yamark --reps 1 --warmups 0
```

The generator can also be run without timing formatters:

```sh
tools/bench/big.R --out-dir target/bench-big/corpus
```

## Other modes

Directory mode passes the copied corpus root to the formatter once and lets the formatter discover files. The website's directory comparison uses 500 files of about 100 KB each:

```sh
tools/bench/run.py --corpus-shape flow-heavy --invocation directory --files 500 --items 540 \
  --reps 3 --warmups 1 \
  --tools yamark,yamlfmt,prettier,yamlfix,dprint-yaml,deno-fmt
```

The harness does not run multiple formatter processes concurrently. In `directory` mode, any file discovery or internal parallelism comes from the formatter itself.

Run the comparison formatter set explicitly when needed:

```sh
tools/bench/run.py --tools yamark,yamlfmt,prettier,yamlfix,panache-yaml,dprint-yaml,deno-fmt,pretty-yaml,py-yaml12
```

Missing external tools are recorded as skipped. By default, the harness runs 2 measured repetitions after 1 warmup. Use `--reps 5` when you need lower-noise comparison data.

Use `--files` and `--items` to size the generated corpus:

```sh
tools/bench/run.py --files 400 --items 80
```

Use `mixed-node` for the r/py-yaml12-style size-scaling fixture. It writes a single-line JSON document to each `.yaml` file, so JSON-as-YAML inputs are parsed and rendered into formatted YAML instead of starting from mostly preformatted YAML:

```sh
tools/bench/run.py --corpus-shape mixed-node --files 1 --items 32000 --reps 5 --tools yamark,py-yaml12
```

Use the same single large file for the full formatter set:

```sh
tools/bench/run.py --corpus-shape mixed-node --files 1 --items 32000 --reps 5 --tools yamark,yamlfmt,prettier,yamlfix,panache-yaml,dprint-yaml,deno-fmt,pretty-yaml,py-yaml12
```

Run from R:

```sh
tools/bench/run.R
tools/bench/run.R --invocation directory --tools yamark
```

Or source the R runner:

```r
source("tools/bench/run.R")
run_yaml_benchmark()
```

Each default run writes one commit-scoped JSON artifact:

```text
docs/benchmarks/yaml/<commit>-representative.json
```

Generated corpus directories under `target/bench-yaml` are removed after the artifact is written. Use `--keep-corpus` only when you need to inspect the raw generated inputs.

The representative artifact records each workload family's file count and names the family in each input path. Other shapes use `<commit>-<shape>.json`, except `flow-heavy`, which retains its historical `<commit>.json` name. Changing the default therefore does not overwrite or compare against a different corpus. The big-file and published directory comparisons retain their existing corpora.

The artifact contains one result row per selected formatter and invocation. Re-running the same formatter and invocation on the same commit replaces that row while preserving other invocation rows for the commit.

Summarize recent default-path JSON artifacts from R:

```sh
tools/bench/summarize.R --input-dir docs/benchmarks/yaml
```

Or source the reader from R:

```r
source("tools/bench/summarize.R")
results <- read_yaml_benchmark_results("docs/benchmarks/yaml")
```

Use summary filters to compare other benchmark rows:

```sh
tools/bench/summarize.R --formatter "" --invocation directory --corpus-shape flow-heavy --limit-commits 3
```

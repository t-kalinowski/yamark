# YAML Formatter Benchmark

This benchmark measures dirty YAML formatting throughput over one deterministic
generated corpus. Each timed repetition copies the corpus, then invokes the
formatter according to `--invocation`.

## Default Run

When asked to "run benchmarks", use the default benchmark path:

```sh
tools/bench/run.py
```

The default path runs only `yamark` in `per-file` mode. This invokes the
formatter separately for each YAML file, avoids directory-mode internal
parallelism, and is sized to finish in under two minutes on the usual
development machine.

Then present the recent `yamark` per-file comparison:

```sh
tools/bench/summarize.R
```

By default, the summary shows the last five matching commits for `yamark`,
`per-file`, `write`, and the `default` width profile.

## Agent Run

When a coding agent is asked to run benchmarks, use the bounded summary path:

```sh
tools/bench/run.py --agent-summary
```

This runs the same default benchmark, writes the commit-scoped JSON artifact,
and prints only a compact final report with the current row, previous matching
row, delta, artifact path, and log path. Do not open the JSON artifact unless
debugging a failure or checking fields not shown in the summary.

When invoking this from a polling model tool, use a long command wait and a low
output token limit. If the tool returns a running session, poll no more than
every 20 minutes.

## Big Single-File Benchmark

The big-file benchmark generates three deterministic single-file corpora -
`big.md`, `big.yaml`, and `big-with-frontmatter.md`, 4 MB each (the largest
round size every comparison tool accepts: panache refuses inputs over
4 MiB) - and times each formatter's own CLI on each file it natively
supports:

```sh
tools/bench/big.py
```

The default tool set is the native formatter CLIs, each invoked with no
formatting options (`prettier --write FILE`, `panache format FILE`, and so
on). Tools that cache format results (`panache`, `dprint`) run with their
cache redirected into the benchmark work directory and wiped between
repetitions, so every timed run formats from scratch. Lint fixers
(`pymarkdown`, `markdownlint-cli2`) and library shims (`panache-yaml`,
`pretty-yaml`, `py-yaml12`) are not formatters invoked the way a user would
format a file, so they are not in the default set; select them explicitly
with `--tools` when needed.

The front matter file embeds a deliberately unformatted YAML block sized
from the document (5% of bytes, about a third of the lines); override it
with `--frontmatter-yaml-bytes`. For each Markdown file with front matter,
the artifact records a `front_matter` outcome - `rewritten`, `preserved`,
or `removed` - derived from the output bytes. Override all
three sizes for a quick smoke run:

```sh
tools/bench/big.py --target-bytes 12000 --tools yamark --reps 1 --warmups 0
```

The generator can also be run without timing formatters:

```sh
tools/bench/big.R --out-dir target/bench-big/corpus
```

## Other Modes

Directory mode passes the copied corpus root to the formatter once and lets the
formatter discover files. The website's directory comparison uses 500 files of
about 100 KB each:

```sh
tools/bench/run.py --invocation directory --files 500 --items 540 \
  --reps 3 --warmups 1 \
  --tools yamark,yamlfmt,prettier,yamlfix,dprint-yaml,deno-fmt
```

The harness does not run multiple formatter processes concurrently. In
`directory` mode, any file discovery or internal parallelism comes from the
formatter itself.

Run the comparison formatter set explicitly when needed:

```sh
tools/bench/run.py --tools yamark,yamlfmt,prettier,yamlfix,panache-yaml,dprint-yaml,deno-fmt,pretty-yaml,py-yaml12
```

Missing external tools are recorded as skipped. By default, the harness runs 2
measured repetitions after 1 warmup. Use `--reps 5` when you need lower-noise
comparison data.

Use `--files` and `--items` to size the generated corpus:

```sh
tools/bench/run.py --files 400 --items 80
```

Use `mixed-node` for the r/py-yaml12-style size-scaling fixture. It writes a
single-line JSON document to each `.yaml` file, so JSON-as-YAML inputs are
parsed and rendered into formatted YAML instead of starting from mostly
preformatted YAML:

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

Each run writes one commit-scoped JSON artifact:

```text
docs/benchmarks/yaml/<commit>.json
```

Generated corpus directories under `target/bench-yaml` are removed after the
artifact is written. Use `--keep-corpus` only when you need to inspect the raw
generated inputs.

The artifact contains one result row per selected formatter and invocation.
Re-running the same formatter and invocation on the same commit replaces that
row while preserving other invocation rows for the commit.

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
tools/bench/summarize.R --formatter "" --invocation directory --limit-commits 3
```

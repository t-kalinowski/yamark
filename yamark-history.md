# Yamark development history: a source-backed reconstruction

This reconstructs Yamark's development from May through July 2026. It is
not the blog post. It is the longer record from which a blog post can be
written: what I asked for, what the agents did, how the workflows
changed, where my directions changed, and what the repositories show.

The account uses three kinds of primary evidence:

- the Git histories and repository contents of V1, V2, and the current
  public repository;
- the Kata database used for V1; and
- retained coding-agent sessions.

This version is internal source material. It deliberately records hashes
from private repositories, recoverable pre-squash commits, and work that
never reached public `main`. It contains no raw session identifiers or
local paths, but those private-history details should be an explicit
publication choice rather than copied automatically into a public
article.

Git commits use my configured identity. Every Kata issue, comment, and
link does too; nearly all Kata events do. Those records do not, by
themselves, establish whether I or an agent wrote a particular sentence,
issue, or patch. Retained sessions are the best evidence for that
division. Where the only source is my recollection, I say so. Raw
sessions are private: they also contain system instructions, absolute
paths, permissions, and unrelated context. This document quotes or
summarizes only the parts relevant to Yamark.

All exact Git times in the early chronology are shown in Eastern
Daylight Time, the timezone recorded by the original commits. The
review-loop source records were reconstructed in UTC; converted burst
times are explicitly labeled EDT.

## Chronology at a glance

- **May 6:** V1 begins with a 1,416-line specification. The first
  implementation lands 47 minutes later.
- **May 6–8:** Interactive work expands the project from a YAML and
  front matter formatter into Markdown, embedded-language, editor,
  Git-filter, and documentation work.
- **May 8:** Kata becomes the issue store. The first 32 issues are
  loaded, and the one-ready-issue drain loop lands as the 83rd commit.
- **May 8–18:** Fresh sessions drain a growing dependency graph while I
  keep using the formatter, adding work, changing decisions, and
  directing larger migrations.
- **May 13–15:** V1 gains custom source-oriented Markdown and YAML
  implementations behind fast paths.
- **May 17:** The new paths become the normal paths and the superseded
  runtime implementations are removed.
- **May 18:** A late V1 architecture epic and a separate architecture
  review make the remaining structural problems explicit. V1 reaches the
  selected 448-commit endpoint.
- **May 18:** V2 starts in a fresh repository with a large
  specification, an architecture document, Rust scaffolding, fixtures,
  and a small executable draft.
- **May 19–22:** Agents repeatedly compare V2 with the specification and
  land structural implementation slices.
- **May 21–30:** Separate whole-repository review and fix sessions run
  in bursts. The final retained review reports no remaining
  specification gaps for its snapshot.
- **May 30–June 6:** I change the specification and product after that
  result. Agents also run a large measured performance campaign, port
  more surfaces, and revise the source-backed architecture.
- **May 31:** V2 records 291 commits in one day, many from candidate
  optimization, measurement, retention, or reversion.
- **June 1–6:** Product, architecture, editor, memory, benchmark, and
  test work continues while the specification itself keeps changing.
- **June 6:** `SPEC.md` is deleted. Thirty-six more V2 commits follow.
- **June 15:** V2 reaches its selected 593-commit endpoint.
- **June 16–17:** V2's contents are copied into a new public history,
  pruned, prepared for release, and deliberately squashed into a fresh
  root.
- **June 19–July 15:** Daily use produces a smaller series of focused
  formatter and packaging fixes in the public repository.

## Before the first commit

My recollection is that Yamark began because I was repeatedly reading
and editing long LLM prompts stored in YAML and then Python files. I was
working on a variation of Symphony in which a workflow could be written
as `WORKFLOW.yaml` or `WORKFLOW.py` rather than `WORKFLOW.md`.
Rewrapping prose inside YAML scalars and source-language strings was
awkward enough that I wanted a formatter built around those cases.

The local record does not capture how the first V1 specification was
produced. The first Git commit already contains the document, so its
provenance cannot be established from the retained sessions.

## V1 begins with a real specification

The V1 root commit, `0af138c`, was made on May 6 at 13:23:11 EDT. It
contained one file: a 1,416-line, 47,399-byte `SPEC.md`. The initial
name in the document was `yaml12`; two minutes later, after I asked an
agent to choose the name Yamark, commit `b3ffede` renamed it.

The original document was not a casual feature prompt. It already called
for:

- a specialized Rust parser;
- source ranges and reused input slices;
- a compact document representation;
- a printer able to copy unchanged source regions;
- low allocation rates; and
- parallel directory traversal.

It was also not the final product. Its Markdown scope was initially
limited to preserving Markdown bodies while formatting YAML front
matter. General Markdown formatting, recursive code-fence formatting,
embedded Markdown in Python and R, the editor extension, and much of
what later made Yamark distinctive were not yet part of the
specification.

The specification changed almost immediately. A required unchanged-file
cache was revised and then moved to future work within 24 minutes: the
active design would simply reparse each time. I added the YAML test
suite as an acceptance target and pointed agents at the local
`py-yaml12` and `r-yaml12` projects for examples.

At approximately 13:52, I gave the first retained implementation
direction:

> Implement SPEC.md. Spawn subagents where appropriate to do orthogonal
> chunks of work.

The first implementation, `eef874b`, landed at 14:09:53. It added 3,410
lines across 12 files, 46 minutes and 42 seconds after the root commit.
The agent described it as a conservative first implementation. I
rejected that scope in three successive directions:

- “Continue until full compliance.”
- “Continue until full conformance against the yaml-test-suite.”
- “I don't want an MVP. I want to go for the ‘real thing’ right away.”

The initial implementation session used subagents for bounded work. One
worked on Markdown front matter, another on workspace discovery and
writes, while the main session worked on the formatter and integration.
Concurrent edits caused some transient signature and file conflicts,
which the main agent reconciled.

I also corrected the implementation strategy, not just its output. An
early attempt used a general YAML parser as a runtime gate and began
building a test oracle around parser event streams. My directions were:

- Saphyr could be consulted, but must not be a runtime dependency or
  gate on the fast path.
- Formatter conformance should be checked against actual YAML values,
  not by forcing the implementation to reproduce the test suite's event
  stream.
- The project should not grow a hand-written JSON parser and emitter
  merely to run the conformance suite.
- External conformance tests could use Python, `py-yaml12`, and the
  standard JSON module.

The Git history preserves the resulting oscillation. `db11bb4` added
YAML test-suite work. `f44ade3` replaced a `yaml-rust2` syntax gate with
Saphyr. `e89a0b3` then removed Saphyr from the runtime, documented
project workflows, and established the external test-oracle policy and
benchmark infrastructure.

## Eighty-two interactive commits before Kata

Kata and the unattended loop were not present at the beginning. The
first 82 commits were made through interactive work. Immediately before
the drain loop, the repository already had 122 tracked files, including
31 Rust, JavaScript, Python, and R files with 16,656 lines of source.

During those two days, the product moved far beyond the original
front-matter scope. Representative additions included:

- a Markdown AST formatter;
- Quarto-aware Markdown behavior;
- pipe tables, reference links, footnotes, and prose wrapping;
- Markdown embedded in explicitly marked Python and R strings;
- Markdown stored in marked or tagged YAML scalars;
- Git clean and smudge filters;
- formatter directives and canonical modes;
- a VS Code and Positron extension;
- format-on-save support;
- a website and examples; and
- recursive dispatch to format supported fenced regions and code chunks.

The commit sequence makes the expansion visible. `438378f` added the
Markdown formatter. `ccd37c1` made it safer around Quarto constructs.
`ea1d9dd` added embedded Markdown in Python and R. `8cc1971` formatted
marked YAML block scalars as Markdown. `c43a318` added Git-filter
support. `507518a` added the editor extension. `d99a05e` and `98b1133`
added supported Markdown fences and chunks.

The retained sessions show how these features arose. I would use the
formatter, paste a concrete input or bad result, describe the output I
wanted, and ask an agent to investigate. This produced decisions that
were difficult to make in the abstract. For example:

- I first said quoted YAML strings should remain quoted, then
  reconsidered and asked for aggressive simplification when the value
  could be preserved.
- I made prose rewrapping in folded scalars the primary goal and asked
  for real-world examples before implementation.
- I directed multiline flow collections toward expanded block layout,
  then made a specific exception for flow-sequence single-pair entries.
- I said anchors should remain anchors because their presence records
  author intent, even when surrounding layout is normalized.
- I asked that long quoted strings containing `\n` become literal blocks
  when that was the most readable value-preserving representation.

These were product decisions discovered while the product existed. They
were not all derivable from the initial specification.

## Kata becomes the project memory

On May 8 I asked an agent to set up Kata, the issue tracker I was using
at the time. Kata project ID 2 was created at 18:05 EDT with the name
`yamark` and a binding to the intended GitHub repository. Its state
lived outside the Git worktree and was excluded locally rather than
committed.

The first issue, created two minutes later, was “canonical should be a
CLI flag.” In the setup session I supplied ten immediate concerns:

1. Make canonical mode a CLI flag.
2. Define a consistent comment-directive syntax.
3. Document canonical Markdown comment directives.
4. Make semantic rewrite checks opt-in.
5. Build pathological YAML and Markdown corpora.
6. Test Markdown formatting in interpolated strings.
7. Define canonical handling around protected inline spans.
8. Define directive scope syntax.
9. Add readable diff and dry-run output.
10. Revisit Markdown reference handling.

The agent expanded those short directions into issue bodies, questions,
and completion criteria. I then asked it to turn the existing
`yaml-missing-support.md` workpad into an issue hierarchy while
retaining decisions I had marked in the document. Among those decisions
were that flow sequence table targets were not a V1 goal, duplicate-key
schema validation was out of scope, and tag/directive work should
prevent formatter corruption rather than turn Yamark into a schema
validator.

The first 32 issues were loaded before the first drain run. They were
not all handwritten by me. Some were direct transcriptions of my
concerns; others were agent expansions or decompositions of existing
documents. That distinction becomes even less recoverable later because
every Kata issue, comment, and link used my configured identity.

## The one-ready-issue drain loop

At 19:01 EDT on May 8, I asked for a simple script that would keep
invoking `codex exec` while Kata reported ready work. I refined the
contract interactively:

- each fresh session should ask Kata for one ready issue;
- parent or oversized work should be split recursively into smaller
  children;
- the agent should work on one ready leaf, then stop;
- Kata should be updated even when the session found a blocker or only
  decomposed work;
- a nonzero agent exit should stop the outer loop;
- the default maximum should be 100 sessions;
- the driver should record overall and per-session time; and
- each session should have a combined output log.

Commit `fdc9c62`, titled “add ralph loop,” landed at 20:18:16. It was
the 83rd commit and added a 67-line prompt, a 66-line shell driver, and
a 285-line regression test.

The prompt began:

> Work exactly one ready kata issue in this repository, then stop. Do
> not continue to a second issue; the outer drain script starts the next
> Codex session.

Its actual contract was much more specific than “take an issue and close
it.” The session had to:

- initialize Kata and call `kata ready --limit 1 --json`;
- inspect the selected issue and choose an unblocked child when
  appropriate;
- recursively decompose work too large for one session;
- maintain comments, parent links, blocking links, related links, and
  discovered blockers;
- add a failing test through the public API or CLI before behavior
  changes;
- run focused and broader checks;
- update documentation for public changes;
- commit only the intended project changes;
- close an issue only after it was complete, tested, and committed;
- avoid pushing; and
- report the issue, commit, checks, tracker updates, and blockers at the
  end.

After four failed sessions on the same issue, the prompt called for a
separate blocker issue, a stalled annotation, and a future-work document
so the original issue would stop appearing ready.

The shell script used the plain `kata ready --limit 1` output as an
emptiness probe, launched a fresh `codex exec` in the repository, timed
the call, and stopped on failure, no remaining ready issue, or the
configured limit. Five minutes after the loop commit, `cd65148` added
per-session logs under the build directory.

The first invocation began 24 seconds after the loop landed. It did not
leave a retained completed result. The next completed session selected
the list-prose issue and produced `c4c8362` at 20:37.

The driver itself needed several corrections. Early sessions could edit
the worktree but not update the global Kata database or write Git
metadata under the sandbox. At least one agent committed a change but
could not close its issue. Later commits explicitly set the working
directory, clarified sandbox behavior, and eventually launched sessions
with unrestricted repository and tracker access. A June maintenance
patch updated that invocation again so drain sessions retained the
ability to commit.

The intended unit was one issue, one fresh session, one tested commit,
one close. The observed record is looser. Some sessions only decomposed
an epic, found a blocker, audited behavior that was already correct, or
failed before updating Kata. Some issues were reopened. A fresh context
was still the normal unit of execution, but session, issue, and commit
counts cannot be joined one-to-one without a separate provenance record.

There are 325 retained session records whose first user message is the
exact drain prompt and whose recorded working directory is V1. Their
aggregate wall time is approximately 63 hours and 45 minutes. That is a
sum of session intervals, not 64 hours of one serial process: runs could
overlap, and some records contain idle or retry time. From the
drain-loop commit through the selected endpoint, Git contains 366
commits including the loop commit itself. The similar counts do not
justify pairing sessions and commits by position.

## The issue store becomes an execution graph

The V1 Kata project ultimately contained 355 issues, all closed:

- 351 as done;
- two as duplicates; and
- two after an audit found no code change was needed.

It also contained 452 comments across 310 issues and six reopen events
across five issues. Its structure was substantial:

- 99 root issues;
- 153 issues at depth one;
- 93 at depth two;
- 10 at depth three;
- 53 issues with children;
- 302 leaves;
- 256 parent links;
- 321 blocking links; and
- 30 related links.

Only 32 issues existed before the first drain run. The other 323
combined new directions from me, agent-created decompositions, follow-up
defects, audits, and blockers. Kata's author field cannot distinguish
those categories.

Issue creation was bursty because larger goals were converted into
ordered graphs. Examples include a 17-child website epic, an 11-child
fast Markdown epic, two separate fast YAML epics with 10 and 11
children, and an eight-child annotated-AST architecture epic. Kata's
ready ordering did not work the way I expected from priority alone, so
agents encoded order using blocking edges. Epic comments record literal
chains in which each milestone blocked the next and the final child
blocked the parent from closing.

The issue-creation dates make those decomposition waves visible:

| Date (EDT) | Issues created |
| ---------- | -------------: |
| May 8      |             37 |
| May 9      |             19 |
| May 10     |             28 |
| May 11     |             13 |
| May 12     |             15 |
| May 13     |             35 |
| May 14     |             62 |
| May 15     |             52 |
| May 16     |              6 |
| May 17     |             48 |
| May 18     |             40 |

Representative anchors in the issue graph include:

- `evy5`, the first issue, for the canonical CLI flag;
- `hcxd`, the 13-child YAML missing-support tracker;
- `deh8`, the 17-child website improvement epic;
- `v2sp`, the 11-child custom Markdown fast-path epic;
- `w24r`, the 10-child initial custom YAML epic;
- `0dje`, the 11-child effort to make that YAML path the sole runtime
  implementation; and
- `md98`, the eight-child annotated-AST architecture epic near the end
  of V1.

The most common overlapping labels show where the queue spent its
effort:

| Label             | Issues |
| ----------------- | -----: |
| `fast-yaml`       |     76 |
| `markdown`        |     68 |
| `formatter`       |     68 |
| `performance`     |     58 |
| `benchmark`       |     40 |
| `yaml`            |     27 |
| `missing-support` |     23 |
| `Quarto`          |     16 |
| `tests`           |     13 |
| `docs`            |     13 |

These are not mutually exclusive categories. A single issue could be
Markdown, performance, benchmark, and formatter work at the same time.

## What agents did in the V1 queue

The first drain wave, on May 8 and 9, worked through the initial
behavior queue: list wrapping, widths, quote selection, missing YAML
syntax, flow collections, table modes, compact sequences, directives,
and pathological corpora. A typical successful session selected a leaf,
wrote a public failing test, implemented the behavior, ran checks,
updated any affected docs, committed, commented on the issue, and closed
it.

The next phase mixed product, editor, and publication work. On May
10–12, agents decomposed and built the website, revised documentation,
worked through VS Code formatter chaining and Quarto save behavior,
repaired CLI details, and added packaging. I continued to use the
formatter and start interactive sessions during this period. V1 was
never sealed off as a single unattended experiment: the queue drained
while I inspected results, added concerns, and changed direction.

On May 13, performance work led to a larger architectural change. After
benchmarking the current Markdown path and investigating Deno and
Prettier overhead, I asked for a custom formatter that retained Yamark's
behavior while avoiding the existing path. I directed the agent to:

- keep the old implementation as a reference;
- put the new implementation behind `--fast`;
- isolate the new parser in its own file; and
- run both implementations against the same public behavior.

The resulting fast-Markdown epic divided work into CLI plumbing, a
source-slice scanner, directive planning, protected inline wrapping,
lists and blockquotes, front matter and fences, a differential
conformance harness, tables, integration, benchmarks, and documentation.
The first scanner landed in `b20b543`; most of the epic was completed
overnight.

On May 14 I asked for an equivalent investigation of YAML, initially
without implementation. After reviewing the analysis, I directed another
ordered epic using the same reference-path migration strategy. Agents
built a boundary and facade, a scanner and range model, a small semantic
representation, a source-slice printer, flow and scalar formatting,
Markdown-scalar integration, CLI wiring, parity checks, and benchmarks.

The benchmark evidence did not initially support the “fast” label. One
comparison put the new path near 6.66 MB/s and the old path near 56.84
MB/s. I questioned whether the corpus was representative, whether tools
were silently skipping files, and whether the compared paths did
equivalent work. The response was not just a faster function: agents
created another 11-child epic to make the new YAML path own comments,
directives, document markers, embedded formatting, preservation, and the
normal runtime path while improving the benchmark method.

During the same period, the queue continued broadening Markdown
behavior: Pandoc grid, simple, and multiline tables; definition lists;
Quarto chunk options; math; captions; image attributes; list markers;
headings; and template protection.

By May 17, I wanted the dual-path transition finished. Runtime double
checking was moved into test-only semantic harnesses, the new formatters
became the defaults, old selectors and paths were removed, and
documentation was updated to describe one formatter rather than “fast”
and “reference” modes. Commit `2212e9e` made the new engines the
default; `4569634` removed the superseded runtime code.

## Measurement work in late V1

I repeatedly challenged benchmark results instead of treating stored
numbers as self-explanatory. The history records several methodological
failures:

- benchmark corpora on which a formatter silently did no work;
- dirty and already-formatted inputs combined into one result;
- comparison tools receiving different effective work;
- temporary copies placed where ignore rules changed discovery; and
- regressions noticed only several commits after they were introduced.

The resulting benchmark epics moved work copies under `/tmp`, added
sentinels to prove each formatter touched its input, separated dirty
from idempotent workloads, corrected the Prettier baseline, regenerated
artifacts, and added quick per-commit Yamark snapshots. This is
important context for the later V2 performance campaign: the idea that
every attempted optimization should carry measured evidence came from
problems already encountered in V1.

## The late V1 architecture correction

By May 18, the architecture I wanted was more explicit than it had been
at the start. I directed an epic around a single scan and an annotated
AST:

- instrument how many passes and line walks occurred;
- scan once and reuse the result;
- compute range classifications once;
- attach trivia and directives while parsing;
- put formatting choices on planned nodes; and
- consolidate emission around that representation.

The resulting issue graph reached depth three. Commits attached leading
trivia, inline comments, Markdown-scalar directives, preservation
directives, table directives, and support decisions to the parsed
representation. The epic closed shortly after midnight UTC on May 19.

The selected V1 endpoint, `c8096c2`, was committed on May 18 at 21:10:52
EDT. It was the 448th commit in twelve days.

Twenty-two minutes later, I began a retained session with a separate
repository architecture review. The review called V1 substantial but in
need of a new center. It found real parsers and useful AST structures,
but also:

- separate scan, parse, plan, and emission walks;
- normalized full-buffer copies;
- owned scalar strings and other allocation-heavy intermediate data;
- formatting decisions still being made during emission;
- recursive embedded formatting and external dispatch entangled with the
  core; and
- duplication left by the old-to-fast migration.

The recommendation was a controlled replacement of the core while
preserving the behavior and tests. I briefly asked that the findings be
turned into Kata work, then interrupted before implementation and asked
for the direction to be described first.

No retained V1 message says, in one sentence, “abandon this repository
and create V2.” The next repository begins roughly an hour later. The
timing, the review, and V2's explicit instruction not to copy from V1
make the reset a strong inference, but the exact decision is not
directly recorded.

## What V1 was, and was not

At `c8096c2`, V1 contained 335 tracked files and 50,615 lines across
Rust, JavaScript, Python, and R. The Rust source alone was 24,510 lines.
Its three largest core modules were:

| File                   |  Lines |
| ---------------------- | -----: |
| `src/fast_yaml.rs`     | 10,085 |
| `src/fast_markdown.rs` |  5,207 |
| `src/formatter.rs`     |  4,582 |

V1 did not directly depend on Rust's `regex` crate and did not use
`Regex` in its source or tests. It had custom YAML scanners and parsers,
a Markdown block scanner, source-range preservation, and substantial
test infrastructure. My dictated memory of “regexes on top of regexes”
was factually wrong.

The accurate criticism is that the working formatter accumulated several
generations of scanning, parsing, planning, preservation, and emission
logic inside very large modules. The issue queue was effective at
continuing to produce local behavior and bounded migrations. It was less
effective at keeping the whole system in the global shape I eventually
decided I wanted.

V1 did not fail. It became useful enough to drive daily product
discovery. It also produced the tests, examples, performance questions,
feature vocabulary, and architectural dissatisfaction that made V2
possible.

## V2 starts from a prepared first state

My recollection is that, after deciding to build V2, I gave ChatGPT Pro
the V1 tests and desired behavior and asked it to produce a
specification. I then described the parser and emission architecture I
wanted. The retained coding-agent sessions do not cover that work or
establish how long it took.

The V2 root, `967af60`, was committed on May 18 at 22:40:09 EDT, 89
minutes after the selected V1 endpoint. It was a new repository, but not
an empty one. Its first commit contained 30 files and 3,521 lines,
including:

- a 1,203-line `SPEC.md`;
- a 60-line `ARCHITECTURE.md`;
- a compilable Rust project;
- modules for the CLI, config, workspace, source buffer, document model,
  directives, parsers, emission, Markdown, YAML, and plugins; and
- five readable case fixtures with a CLI test harness.

The first line of the specification described itself as “extracted from
the current `yamark` source” and as a record of “current behavior and
inferred product intent.” It also said it should be refined before being
treated as final. That wording is more precise than my recollection that
the document came from dumping the test suite alone. The retained record
cannot establish what I gave ChatGPT Pro or how long that generation
took; it can establish what entered Git.

The root architecture already stated the desired pipeline:

1. Scan the original UTF-8 source into a `SourceBuffer` with line spans
   and line-ending information.
2. Parse forward over ranges in that buffer.
3. Consume directives and assign state while parsing.
4. Choose an `EmitPlan` as each node is created.
5. Patch earlier nodes only when a whole-file directive requires it.
6. Emit mechanically into one output string.

Nodes were meant to keep spans rather than copies of source text. Nested
documents, including front matter, fenced content, YAML Markdown
scalars, and Python or R strings, were meant to keep referring into the
original buffer. The main tests were meant to exercise the compiled CLI.

The first retained V2 session began 36 seconds after the root commit, so
it did not produce the specification, architecture document, or
scaffold. Its first direction was:

> Implement the SPEC.md here. Do not stop until all the features
> described in SPEC.md are implemented.

The agent added a broad set of public CLI tests and a first
implementation across the existing modules. After it reported back, I
said “commit.” Commit `3feacdd`, “Implement SPEC formatter behavior,”
landed at 23:14:54, roughly 34 minutes after the root. It changed 11
files, added 2,574 lines, removed 199, and created a 511-line
`tests/spec_cli.rs`.

This was a fast first pass, not a finished implementation. Later reviews
found gaps across nearly every major surface.

## What was and was not ported from V1

Immediately after the broad implementation, I explicitly asked the agent
to look in V1, especially at its test suite, benchmarks, and website,
and decide what should be ported. I asked for a high-level port plan and
the first slice. That produced an external Python smoke-test harness and
focused CLI suites in `a428e32` and `ed77f57`.

The boundary tightened when an agent began using V1 product code as a
source during a specification-alignment task. I stopped it, asked
whether it was copying from V1, told it to discard that work, and
replaced the instruction with:

> Do not look in ../yamark, do not copy anything from there. Only
> compare SPEC.md and the source code here.

The correct account is not that V2 never inspected V1. Selected tests,
harnesses, benchmarks, and later the website and editor extension were
deliberately ported. The product implementation was then developed under
an explicit rule against copying the V1 core, and I intervened when an
agent crossed that line.

## May 19: designing the parser while it was being built

The day after the root commit was a high-touch architecture phase rather
than an unattended loop. I reviewed proposed structures, asked how they
handled specific YAML forms, and supplied detailed constraints about
parsing, representation, and emission.

Among the retained directions were:

- build a real forward YAML parser and an AST of nodes plus trivia,
  rather than retaining a line-oriented heuristic formatter;
- keep scanning through skipped regions so later directives and document
  structure remain known;
- represent multiline scalars as source-backed fragments when
  indentation, folding, or separators make one contiguous span
  insufficient;
- keep durable semantic and presentation information separate from
  transient output choices;
- make recursive emission take explicit context, indentation, and active
  options;
- preserve parsed tags, anchors, comments, and spelling information
  without copying the full source into nodes; and
- support compact forms, including inline block sequences of mappings
  when width permits.

The Git sequence follows those conversations. Agents added
directive-aware AST tracking, inline-comment metadata, flow collections,
block-mapping sequence items, multiline flow parsing, and progressively
broader parser coverage. Commit `6f94939` added a separate YAML emission
design document.

I also asked for public tools that would make losslessness observable:

- `dump-ast`, which serialized the complete YAML representation and
  trivia;
- `emit-ast`, which read that representation and reproduced the input;
  and
- round-trip checks against the YAML suite used by `r-yaml12`.

Those directions produced `adfbfe1`, `28bcf2c`, `c1e7050`, and
`80891a4`. The tools were useful constraints during parser development.
They were not permanent product commitments, as later history shows.

I designed benchmark constraints during this phase too: representative
and simple corpora, equivalent serial invocation, artifacts tied to the
formatter and commit, and open-source comparisons. When results looked
implausibly favorable, I asked whether the benchmark had forced every
tool to do real work. The first V2 YAML benchmark harness landed in
`26f0a6f`.

By early May 21, the repository had a much more concrete parser and
testing surface than the initial broad implementation. The architecture
was not merely selected by an agent from `SPEC.md`; it had been refined
through repeated questions about how source spans, trivia, semantics,
directives, nesting, and emission would actually work.

## Before the automated loop: repeated specification-alignment slices

The workflow then became more repeatable. A common prompt asked a fresh
agent to inspect only `SPEC.md` and the V2 repository, identify the most
important structural mismatch, implement one coherent slice, test it,
commit, and stop. The prompt explicitly favored architecture and
data-model corrections over isolated output special cases.

This produced a sequence of parser, directive, emission, and test
commits before the later driver was checked in. It is useful to
distinguish these one-slice invocations from the formal whole-spec
review/fix protocol. Both used fresh sessions and the specification, but
the earlier prompt let the same session choose and implement its own
next slice. The later protocol assigned review and implementation to
separate sessions.

## The whole-spec review/fix driver

The formal loop lived in `implement_spec.py`. Session evidence shows it
running on May 21 and 22 while still untracked. It did not enter Git
until commit `b57b4ca` on May 28, after the loop had already been used.

For every call, the driver started a fresh `codex exec --yolo` and
embedded the full current `SPEC.md` in the prompt. Reviewers and fixers
shared no conversation. Their common state was the repository, the
specification, and, for a fixer, the reviewer's structured findings.

The review result had two fields:

```json
{
  "spec-completed": "yes | no",
  "findings": "string | null"
}
```

The reviewer was instructed to compare the entire repository with the
entire specification. It could inspect code, run checks, and improve
tests, fixtures, harnesses, or test configuration. It could commit only
test-surface changes. It was forbidden from fixing product code. A `no`
result had to contain concise product-code gaps that a separate session
could act on.

The fixer received those findings as starting points rather than a
complete checklist. It was told to reread the specification, inspect the
repository, fix all remaining gaps it could find, prioritize
architecture before the long tail of edge cases, run the relevant
checks, and commit its work.

The code also contained a milestone implement/verify mode. The retained
timing series discussed here comes from the whole-spec review/fix mode.

In practice, the role separation held. Reviewers spent most of their
time reading source, probing the CLI, and running tests. No reviewer
product commit appears in the matched runs. Fixers did the edits,
formatting, tests, and commits.

These approximate totals classify retained tool calls by command family;
fixer commits are associated using commit hashes in the session results
and the corresponding Git state. The 89 review calls made approximately
6,747 tool calls, with a median of 77 per review. About 5,219 were
source or file inspection; the reviews also ran roughly 90 Cargo-test
commands, 97 external-test commands, and 197 Git-inspection commands.
The 75 fixer calls made approximately 5,788 tool calls, also with a
median near 76. They included about 3,175 inspection calls, 744
Cargo-test commands, 717 Git-inspection commands, 160 formatting or lint
commands, 95 commit commands, and 71 external-test commands.

The most frequently touched paths in associated fix commits were
`tests/spec_cli.rs`, `src/core/yaml.rs`, `src/core/markdown.rs`, and
`src/core/wrap.rs`, followed by embedded-source, emission, document, and
directive modules.

## What the reviews found

The first retained reviewer returned a broad inventory. It found missing
Markdown block forms, Quarto and Pandoc behavior, preservation cases,
inline syntax, and footnote handling. It found incomplete directive
errors and scope rules; YAML scalar, flow, tag, and trace behavior;
naive Python and R target recognition; incomplete external-formatter
paths, preambles, and diagnostics; and a CLI diff that was not a proper
unified diff.

The corresponding fixer produced `6eebcfa`, “Close formatter spec gaps
from review.” It added public regression coverage and changed directive
validation, template inference, embedded configuration, YAML Markdown
scalars, indentation rejection, external paths and preambles, and diff
headers.

As the loop continued, findings narrowed. Later reviews spent time on
details such as:

- preserving an entire external-formatter target when a formatter was
  skipped or unavailable;
- moving Markdown and YAML choices out of final emission;
- measuring Unicode display width correctly;
- preserving closing indentation around embedded source strings;
- aligning YAML table closing braces before trailing comments;
- recognizing alternate Python and R raw-string forms;
- preserving punctuation adjacent to footnote references; and
- applying canonical emphasis inside GFM and Pandoc table cells.

The final fix, `1caeba3`, added public CLI regressions for canonical
emphasis in table cells, changed the wrapper, ran focused and full
tests, and committed in 2 minutes 49 seconds.

## How the loop actually ran

The retained inventory contains 164 matching calls: 89 reviews and 75
fixes. They ran in bursts, not as one uninterrupted process:

| Burst (EDT)               |    R |    F | Outcomes                         |
| ------------------------- | ---: | ---: | -------------------------------- |
| May 21 22:20–May 22 01:41 |   13 |   12 | 13 `no`                          |
| May 22 08:18–11:19        |   10 |   10 | 10 `no`                          |
| May 28 18:06–18:15        |    1 |    0 | unrelated diff review            |
| May 28 21:15–May 29 04:41 |   23 |   21 | 23 `no`                          |
| May 29 07:33–10:05        |   12 |    8 | 8 `no`, 3 aborted, 1 `yes`       |
| May 29 11:17–18:07        |   21 |   16 | 17 `no`, 2 aborted, blank, `yes` |
| May 29 19:16–May 30 06:04 |    9 |    8 | 8 `no`, 1 `yes`                  |

There was a six-day pause between May 22 and May 28. Direct interactive
work, specification edits, and one separate Kata/Ralph-style step
experiment also appear among the bursts. V2 therefore combined several
workflows rather than one homogeneous unattended run.

For the timing comparison, 73 completed `no` reviews were followed
immediately by fixes. Seventy-one fixes completed and two aborted. The
two aborted fixes are excluded from the fix medians; unmatched, seed,
aborted-review, and terminal calls are treated separately.

| Portion of paired sequence | Median review | Median fix |
| -------------------------- | ------------: | ---------: |
| First 10 pairs             |        4m 25s |    10m 33s |
| Pairs 11–20                |        7m 35s |    10m 01s |
| Pairs 21–30                |        9m 09s |     9m 30s |
| Pairs 31–40                |        8m 59s |     9m 56s |
| Pairs 41–50                |        8m 36s |     6m 09s |
| Pairs 51–60                |       10m 32s |     7m 39s |
| Pairs 61–70                |        8m 50s |     5m 15s |

The ten-pair rolling review median first exceeded the fix median in the
window covering pairs 25–34. That supports a real change in the balance
of work: reviews generally became more expensive relative to fixes. It
does not support my remembered picture of two smooth, monotonic lines
crossing once.

Across all matching calls, review wall time sums to approximately 20.3
hours and fix wall time to approximately 13.0 hours. The median review
was about 8 minutes 40 seconds; the median fix about 9 minutes 10
seconds. As with the V1 aggregate, these are session intervals and can
include overlap or idle time.

## Three `yes` reviews, not one

Reviewers returned `spec-completed=yes` three times:

1. May 29 at approximately 10:05 EDT.
2. May 29 at approximately 16:44 EDT.
3. May 30 at approximately 06:04 EDT.

Work resumed after the first two results because I or later sessions
changed the target, found another behavior to decide, or added
requirements. A `yes` was a claim that one repository snapshot matched
one specification snapshot. It was not an irreversible project state.

The last terminal review began at `2026-05-30T03:11:49Z` and ended at
`10:04:09Z`. That is 6 hours, 52 minutes, and 20 seconds of wall time.
Its own completion event reports 3,335.743 seconds, or 55 minutes and 36
seconds, of task duration. Long gaps appear in the session record. It is
accurate to say the subprocess remained open for nearly seven hours and
that its internal task duration was about 56 minutes. It is not accurate
to call this seven hours of continuous review reasoning.

That reviewer reported no remaining product-code gaps. It ran the Rust
suite and the external smoke and CLI suites and changed no files. The
immediately preceding fix took 2 minutes 49 seconds, not roughly one
minute as I remembered.

## Product decisions kept changing the target

The strongest evidence against treating the specification as a complete
upfront design is not an abstract argument. It is the commit sequence.

### Scalar spelling

A ported golden-suite comparison exposed a difference between V1 and V2.
V1 would simplify a safe quoted string; V2 preserved its quotes under
the then current specification. I had the agent check in the golden
suite, inspect the differences, and record decisions. I then changed the
specified policy in `31fd4cf`:

- use an unquoted form when it is safe and preserves the YAML value and
  type;
- quote only when plain spelling would be ambiguous or invalid;
- choose the quote style requiring less escaping; and
- use block style for suitable multiline values.

Implementation followed 13 minutes later in `baef729`.

### Layout intent after the terminal review

The terminal reviewer returned `yes` at 06:04 EDT on May 30. Less than
five hours later, `5fe982e` added a new rule to the specification: an
unmatched typed `[` or `{` could request flow-style collapse, while
physical newlines in a flow collection could request expanded layout.
Ninety lines of tests were added with the rule. `fd40b2a` implemented it
36 minutes later.

This was not a missed item from an unchanged checklist. It was a new
product decision made after the loop had matched the previous checklist.

### AST commands

The `dump-ast` and `emit-ast` tools I had requested during parser
development were later written into the specification as supported
behavior. On June 2 I decided they did not belong in the product. Commit
`0ef45d9` removed the commands, serializers, loaders, tests, and
round-trip machinery, deleting 3,083 lines. The specification remained
stale until its later deletion.

### Concrete versus semantic representation

The early parser work emphasized a lossless concrete AST capable of
reproducing all source bytes. Performance work later moved the normal
formatting path toward a smaller semantic representation with
source-backed information. `c400f94` introduced the semantic YAML
formatting path, `6ad3de9` records borrowed-source AST work, and
`d61d6e8` records source-lifetime emit plans. Losslessness remained
valuable for development and selected tests, but carrying the full
concrete representation through every normal format became less
attractive.

### Wide Markdown tables

After `SPEC.md` was gone, I changed another visible behavior. The old
rule stopped aligning pipe tables beyond a width threshold. I found that
I preferred aligned tables even when they were wide, and `1fb3d55`
removed the fallback on June 11.

## Performance work after the final review/fix run

The review/fix loop was not the end or even the majority of V2's Git
history. There are 468 commits after the final fix commit, out of 593
total on `main`.

On May 30 and 31, work shifted into a benchmark-gated performance
campaign. I asked for the largest architectural improvements first,
required measurements before retaining changes, and wanted rejected
approaches logged so another agent would not repeat them. I also
challenged suspicious benchmark wins, asked for true single-file and
directory comparisons, pointed agents to the R and Python yaml12
benchmark setups, and suggested starting from single-line JSON so
competitors would be forced to parse and reformat rather than merely
copy input.

The resulting Git history is unusually literal. May 31 alone contains
291 commits by their original dates. A subject-based classification
finds roughly:

- 70 commits beginning `Record rejected`;
- 42 explicit reverts;
- 69 other benchmark-named commits; and
- 110 code, plan, tooling, documentation, or other commits.

The exact grouping is only a subject heuristic, but the experimental
pattern is clear: propose a candidate, run the public tests and
benchmark, retain it if the wall-time evidence improved, otherwise
revert it, and record the result.

Agents tried direct flow emission, delayed table and scalar rendering,
buffer reuse, scanner changes, semantic fast paths, template prefilters,
width caches, directory setup reductions, and many smaller hot-path
changes. Some changes reduced allocation counts but made elapsed time
worse and were removed. Others survived only after comparison with an
adjacent no-code baseline because the benchmark itself drifted.

Representative retained changes include scanning flow plain-scalar
safety once in `d3e3a5e`, bypassing Rayon setup for one-file formatting
in `7319911`, and reusing cached preservation decisions in `c230a91`.
Other candidates reduced allocations but lost elapsed-time benchmarks.
For example, direct inline scalar emission in `6d6c6b7` was reverted by
`714234a`. The checked-in performance plan records the adjacent
baselines, test commands, and benchmark results for these decisions.

The performance plan grew into an agent-readable record of which ideas
had already failed and under what measurements. A branch named
`borrowed-source-ast` remains as a pointer to the `6ad3de9` milestone;
that commit is also an ancestor of `main`.

This was a different agent workflow from whole-spec review. Its unit was
a measured hypothesis, not a specification finding. It depended on
benchmark construction, profiling, candidate changes, retain-or-revert
decisions, and my interpretation of numbers that sometimes looked too
favorable to be credible.

## Porting, memory work, and product work after May 31

The repository continued expanding after the performance burst. Agents
ported the VS Code and Positron extension from V1 in `0d1164a`, adding
2,596 lines of implementation, tests, packaging, and documentation. I
then worked through actual formatter chaining with Python, R, Quarto,
and the Quarto extension rather than treating the port as finished when
it compiled.

Other June work included:

- richer Markdown inline and block syntax;
- IPython magics in Python fences;
- semantic YAML and source-backed lifetime work;
- lower YAML formatter memory usage;
- deferred Markdown and YAML rendering;
- website and benchmark pages;
- benchmark peak-RSS reporting;
- restoration of YAML-suite, semantic, and golden external coverage;
- Git-filter setup and teardown; and
- repeated documentation and benchmark revisions.

The specification changed alongside this work. Starting from 1,203
lines, it was edited 18 times and reached 1,355 lines before deletion.
It gained or revised rules for source-string targets, indentation, hard
breaks, scalar spelling, layout intent, reference links, performance
diagnostics, symlinks, IPython syntax, paragraph spacing, folded prose,
and other behavior.

On June 1, `c291e5c`, “Update SPEC to match current behavior,” changed
the document toward describing the implementation that now existed. This
is evidence of movement in both directions: code was being aligned to
the specification, while the specification was being aligned to
discoveries and decisions in code and use.

Planning artifacts were retired in stages. The YAML emission design
document was deleted on May 30. Several architecture and planning files
were consolidated or removed on June 1. `implement_spec.py` was deleted
on June 5. `SPEC.md` itself was deleted on June 6 in `3b1f48e`. Git
proves the sequence but does not preserve one explicit statement of why
I deleted the specification. The conservative conclusion is that the
document and its driver were no longer the active way the project was
being developed.

## Thirty-six commits after `SPEC.md`

V2 continued for another 36 commits after the specification was deleted.
They included:

- public documentation corrections;
- ordinary review findings;
- front-matter delimiters and mixed-case extension handling;
- contextual diff fallback fixes;
- CommonMark autolinks;
- Git-filter installation and removal;
- YAML `md` tags;
- the wide-table decision;
- benchmark and reporting work;
- Markdown wrapping performance;
- standalone f-string placeholder preservation; and
- folded-scalar trailing blank lines.

The V2 endpoint, `6cba8f3`, was committed on June 15 at 14:30:37 EDT
with the subject “Move CLI help to dedicated page.” `main` contained 593
commits.

At that point the repository contained approximately 27,346 lines of
Rust under `src`, 14,071 lines of Rust tests, 2,665 lines of Python
external-test code, the editor extension, a Quarto website, benchmark
tooling and artifacts, and YAML-suite, golden, semantic, CLI,
pathological, and performance coverage.

V2's history is therefore not “the loop ran until the spec was done.” A
more complete sequence is:

1. A prepared specification and scaffold made a broad first
   implementation possible in roughly half an hour.
2. I spent a high-touch period refining the parser, source
   representation, emission model, observability, and test boundaries
   with agents.
3. Repeated one-slice sessions and then separate review/fix sessions
   drove broad convergence against a moving document.
4. Use and golden-suite comparison changed product rules even after
   `yes`.
5. A large measured performance campaign changed the normal architecture
   and recorded many failed approaches.
6. Porting, memory work, editor integration, documentation, and ordinary
   use continued after the specification ceased to be active.

## Moving V2 into a public repository

The current public repository is a content descendant of V2 but
deliberately not a Git descendant.

The earliest recoverable root of the replacement repository is
`b93420a`, made on June 16 at 09:48 EDT. Compared with the V2 endpoint,
it omitted `PENDING_WORK.md` and one website GIF, and added Python cache
and environment entries to `.gitignore`. Every other shared file has the
same blob hash. That tree identity is stronger evidence of the
transition than recollection alone.

Untimestamped shell history corroborates the operation: the two old
directories were moved under a prototype directory, V2 was copied back
to the public `yamark` path, and its `.git` directory was removed. Shell
history does not provide trustworthy time or working-directory context
for every command, so the Git trees remain the main evidence.

The recoverable pre-public chain was:

- **June 16 09:48, `b93420a`:** new root copied from the V2 endpoint.
- **June 16 10:16, `36f7117`:** amended root after deleting 100 obsolete
  YAML benchmark artifacts.
- **June 17 12:49, `9b05bb5`:** added `--wrap paragraph` after I tried
  Yamark on my own `AGENTS.md`.
- **June 17 13:30, `1c69398`:** removed `yamark-next`, `yamark2`, old
  paths, and other lineage markers.
- **June 17 13:38, `544bf6d`:** squashed the chain into the root
  `initial public release`.
- **June 17 14:32, `48c131d`:** prepared the repository for public
  release.
- **June 17 14:35, `d60f32c`:** amended release preparation for intended
  Python packaging.
- **June 17 17:17, `609b4a3`:** final parentless root, `Initial commit`.

The squashes were explicit directions from me. I asked to squash all
commits into an initial public release, asked that old “next” or V2
references be removed so the repository looked like a new product, and
later asked for “a simple clean commit” as the start of public history.
The current root's tree is identical to the recoverable
release-preparation tree at `d60f32c`. The missing ancestry was
intentional, not an agent accidentally losing it.

The paragraph wrap mode is a small but characteristic bridge between V2
and public use. I tried to format a real `AGENTS.md`, found that
ordinary column wrapping was not the behavior I wanted for
one-line-per-paragraph text, and asked for a `paragraph` mode. The agent
added CLI, front-matter, directive, test, and documentation support
before the history was squashed.

## Preparing and publishing the public repository

I first asked whether the copied repository was ready to make public.
The audit found a missing root license, placeholder package metadata,
development-oriented README language, no public CI, extension packaging
assumptions, and publication gaps. I then said, “Make all the fixes,”
and asked the agent to use subagents to keep the audit work separated.

Agents added:

- a root MIT license;
- a public README with installation, usage, and development commands;
- Cargo repository, homepage, and description metadata;
- Maturin configuration for building the Rust CLI as a Python wheel;
- GitHub Actions CI;
- VS Code Marketplace metadata;
- an extension default that resolves `yamark` from `PATH` rather than
  assuming a bundled executable;
- corresponding extension and website documentation;
- valid social-card image references; and
- public regression tests for metadata, workflows, and documentation.

The release-preparation session recorded a red/green sequence for these
repository checks. Verification included Rust formatting, Clippy, all
Rust tests, the serial external suite, extension tests, a local wheel
build, installation of that wheel followed by `yamark --help`, and a
scan of tracked files for likely secrets.

Before publication, the old private repository name redirected to V1. I
accepted that creating a new repository under the original name would
end that redirect and told the agent to publish.

The public GitHub repository was created at `2026-06-17T21:20:00Z` with
`609b4a3` as its root. V1 and V2 remained private as of July 15.

Local success did not imply clean hosted CI. The fresh runner exposed
missing declarations and produced four immediate follow-up commits:

- `3590b79` fixed Clippy formatting arguments;
- `f458d89` installed R;
- `a9a6866` installed required R packages; and
- `03407b8` installed Ruff.

I next asked whether the website was publishing and, if not, to make it
do so. The agent added a Pages workflow, enabled Pages, observed a
missing R Markdown dependency on the first run, fixed it, and got the
site published.

I then asked for a GitHub release workflow. The agent added
tag-triggered builds for Linux x86-64, Windows x86-64, macOS Intel, and
macOS arm64, plus checksums. The first `v0.1.0` release failed because
the release job had not checked out a Git repository before calling
`gh release create --verify-tag`. Commit `b390896` added checkout; the
tag was moved and the next run published four native archives and
`SHA256SUMS`.

The public evidence supports CI, Pages, and a native GitHub release. It
does not support a claim that Yamark was published to PyPI or the VS
Code Marketplace.

## Public development driven by actual files

The public history is much smaller than either prototype history. Its
later changes continue the same use-and-correct pattern.

On June 19, I found that an R block in a real Python script was not
being formatted. The agent found that a one-line triple-quoted module
string was incorrectly swallowing later directives and that `# fmt: r`
was not accepted for Python source strings. I also rejected a misleading
fake regression test that merely uppercased `abc`; the replacement test
used an R-like spacing change. Commit `9c93f10` added the directive,
scanner correction, dedent/reindent behavior, public tests, and docs.

After installing the extension, I encountered a Node deprecation
warning. `7834365` suppressed that warning only around the VS Code CLI
installation and added extension tests.

On June 26 I supplied a spaced Quarto image-attribute example and
required any rewrite to remain valid Quarto. `b5506d7` added a public
regression and the agent also checked the output with Pandoc and Quarto.
I then supplied a multiline raw HTML tag that was unsafe to rewrap;
`cb0b79c` preserved it through the closing `>`. A third example
contained two spaces inside a Markdown heading. `a6a48ef` routed heading
text through protected-span-aware whitespace normalization.

On July 7, I found a real semantic regression: wrapping a long YAML
scalar collapsed repeated spaces. I restated the invariant that Yamark
must not change the parsed YAML value unless an explicitly requested
external formatter is responsible. The agent added failing public and
`py-yaml12` semantic tests, confirmed them red, changed wrapping to
break only at isolated ASCII spaces, and committed `a1289e8`.

This is evidence against the earlier broad claim that V2 had no
regressions. The narrower, supported observation is that these later
regressions were local enough to fix with focused public tests and
patches rather than another architectural reset.

On July 8 I supplied a multiline brace-attribute example and said it was
unsafe to rewrap. The agent wrote a regression and implementation, but
the change sat uncommitted for a week. On July 15 I asked an agent to
inspect the accumulated files, omit anything incoherent, and split
commits where appropriate. It made three commits:

- `81146f1`, a Python-wheel installation smoke test first written June
  18;
- `ca23973`, a large-file benchmark artifact generated June 21; and
- `95cef0f`, multiline brace-attribute preservation from July 8.

As of July 15, 2026, the public `main` has 18 commits and ends at
`95cef0f`. Its latest CI and Pages runs passed.

## Work that did not reach public `main`

Not every prepared publication feature was finished.

On June 18, I asked for `uv tool install yamark` from PyPI. An agent
prepared cross-platform wheel and source-distribution jobs, Trusted
Publishing, README instructions, and an installation smoke test. Only
the smoke test was later committed. The workflow and README work remain
in a local stash. Current `main` proves local installation from a built
wheel; it does not publish the package to PyPI.

A June 21 experiment added `rumdl` to benchmark tooling and website
presentation. The generated result artifact eventually reached `main`,
but the tooling and website changes remain in the same local stash. The
public website does not currently present `rumdl` even though the
checked-in artifact contains its measurement.

Local stashes and unreachable commits are private and pruneable
evidence. They are part of this reconstruction only because they explain
files that later appeared in public commits; they should not be
described as released work.

## What the record says about direction and execution

The repository cannot be described accurately as code I personally
typed, or as a product an agent autonomously inferred and delivered.

My recorded actions included:

- supplying or importing the specifications;
- rejecting MVP scope;
- choosing runtime and test-oracle boundaries;
- giving concrete formatting examples and deciding ambiguous behavior;
- designing the Kata drain contract and the whole-spec review/fix
  separation;
- defining the parser, source-range, trivia, and emission architecture
  in interactive detail;
- deciding what could be ported from V1 and enforcing a boundary around
  the product code;
- challenging benchmark validity and deciding which performance evidence
  was credible;
- changing the specification after reviewers had matched it;
- deciding when development-only surfaces should be removed;
- using the formatter on real files and reporting the resulting
  failures; and
- deciding how the public history, packaging, CI, website, and release
  should be presented.

The agents' recorded actions included:

- writing most of the patches;
- spawning bounded subagents for independent investigations or modules;
- expanding goals and documents into issue hierarchies;
- selecting ready leaves and maintaining the Kata graph;
- adding public regression tests, often confirming them red first;
- inspecting whole repositories and producing structured review
  findings;
- implementing, formatting, testing, benchmarking, committing, and
  closing work;
- trying and reverting performance candidates;
- porting approved infrastructure and integrations; and
- following hosted CI and release failures through to passing runs.

The division is not perfectly recoverable. Git commits and every Kata
issue, comment, and link use my configured identity, some sessions are
missing, and agent-created issues and prose can carry my author
metadata. The sessions show that agents wrote most patches while I
continued making architecture, product, evaluation, and workflow
decisions.

## Recordkeeping that would have made this reconstruction easier

Neither loop wrote a durable event stream designed for later analysis.
Git, Kata, session logs, benchmark artifacts, and shell history had to
be joined after the fact.

For each unattended call, I would now record at least:

- workflow name and role;
- sanitized prompt version or prompt hash;
- starting commit and dirty-state check;
- issue or review attempt identifier;
- start and end time;
- process exit and structured result;
- findings or issue updates;
- resulting commit; and
- checks run.

That record should be written by the driver, not reconstructed from
agent prose. It would make aborted calls, unmatched reviews, no-commit
sessions, and overlapping wall time explicit.

The raw agent sessions should still not be published wholesale. They
contain system and developer instructions, permissions, local paths, and
unrelated context. A public evidence archive can instead preserve the
faithful Git histories, exact workflow prompts and drivers, a sanitized
Kata export, derived timing data and extraction method, a
commit-to-event index, and hashes for the private source records.

That archive has been planned but does not yet exist. V1 and V2 remain
private, and the current public root intentionally hides their ancestry.
The local publication prompt is an instruction for future work, not
evidence that the archive has already been published.

## Commit anchors

| Event                      | Commit                                     |
| -------------------------- | ------------------------------------------ |
| V1 initial specification   | `0af138c4057d2f36730de81dba681890f4e3c34f` |
| V1 initial implementation  | `eef874b72aaf2590171a9cddd2ea944fa68107ee` |
| V1 issue-drain loop        | `fdc9c6210815c08726d2c55e0f4d08c47c987dce` |
| V1 selected endpoint       | `c8096c24e11a3064ff65ec028a13954ac6807c0d` |
| V1 drain maintenance       | `1487b1cdfe908462c64725fdbd555b521c75e9c6` |
| V2 initial scaffold        | `967af60352cf6822a583c327bed412eba1681b4a` |
| V2 broad implementation    | `3feacdd94d5dd348659da534cb04f5d870eae030` |
| V2 review/fix driver       | `b57b4ca3ad03c39babe6d39030973ad5d3185b87` |
| V2 final loop fix          | `1caeba304a4f4a31f2ada5a81c7cef9a8494103a` |
| V2 layout-intent change    | `5fe982e4d74ad50c4ed0dcd591805e2c0ae6d004` |
| V2 AST commands removed    | `0ef45d91f324c542beac431dc97d80e63778d8f9` |
| V2 specification deleted   | `3b1f48edafae3b19604c5b5efadf013625623e0a` |
| V2 selected endpoint       | `6cba8f362bbe76768e8f3062ef7c311588bb0e20` |
| First recoverable new root | `b93420a9fc256423ca1109a738b6ee307c37018b` |
| Pre-squash release tree    | `d60f32c1eadd0ab0b031078b5adb93a98ef19083` |
| Current public root        | `609b4a39fb4bc4b6c507419ee19eeea19fd36a13` |
| v0.1.0 tag target          | `b390896`                                  |
| Public endpoint used here  | `95cef0f`                                  |

## Claims this history does not support

- **V1 began as a tiny prompt followed immediately by Kata.** It began
  with a large spec and 82 interactive commits before the drain loop.
- **Every Kata issue was written by me.** Agents expanded, decomposed,
  and created much of the issue graph, and the database uses one
  configured identity for every issue, comment, and link.
- **V1 was a regex pile rather than a parser.** It had large custom
  scanners and parsers and no direct Rust regex dependency.
- **One drain session always produced one issue and one commit.** That
  was the intended contract, not a perfect invariant.
- **V2 began as an empty repository.** Its first Git state already
  contained a large spec, architecture, scaffold, and tests.
- **V2 never used V1.** Approved test, benchmark, website, and editor
  material was ported; copying V1 product code was later explicitly
  forbidden.
- **The review/fix loop ran continuously for days.** It ran in bursts
  with a six-day pause and direct work between bursts.
- **Review and fix times followed two clean crossing lines.** The
  aggregate direction exists, but individual timings are noisy.
- **The final review involved seven continuous hours of reasoning.** Its
  wall interval was nearly seven hours; its own task-duration event
  reports about 56 minutes.
- **`spec-completed=yes` meant the product was finished.** It occurred
  three times, and product and specification changes followed each
  snapshot.
- **V2 had no regressions.** At least one semantic YAML regression was
  found in later daily use and fixed with a public test.
- **The public repository preserves the development histories.** Its
  root was intentionally recreated and squashed.
- **Yamark is published to PyPI or the VS Code Marketplace.** The
  current evidence supports native GitHub releases, CI, Pages, and
  public extension source, not those two distribution channels.

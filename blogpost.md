# Yamark: building a formatter with long-running coding agents

Like any curious person in this space, I have been experimenting with
LLM-assisted coding. In particular, I have been exploring long-running,
unattended tasks: setups where agents work for hours or days without
human oversight or intervention.

One of those experiments produced something I now use every day, so I am
writing up what happened.

[Yamark](https://github.com/t-kalinowski/yamark) is a fast YAML and
Markdown formatter written in Rust. It formats YAML, Markdown, YAML
front matter, Markdown in explicitly marked YAML scalars, marked
Markdown strings and comments in Python and R, and `#|` YAML comment
blocks. It formats YAML fences and recursively formats Markdown fences,
and can send other fenced languages to tools such as Ruff, Air, and
Prettier. A `# fmt: table` directive aligns sequences of flow mappings
into readable tables. It also ships with a VS Code and Positron
extension.

“Fast” is measurable here. In the checked-in v0.1.0 benchmarks on an
Apple M4 Max, Yamark formatted a generated 4 MB Markdown document in 115
ms and a generated 4 MB YAML document in 83 ms. The next-fastest tools
in those runs took 338 ms and 177 ms, respectively. On the generated
500-file, 50 MB YAML corpus, Yamark took 123 ms; the next-fastest
formatter took 2.1 seconds. The
[benchmark page](https://t-kalinowski.github.io/yamark/benchmarks.html)
records the corpus construction, tool versions, and method.

Yamark grew out of a mild frustration. I kept looking at LLM prompts
that first lived in YAML files and then in Python files. I was—and, in
Mitch Hedberg fashion, still am—working on a variation of Symphony in
which `WORKFLOW.md` becomes `WORKFLOW.yaml` or `WORKFLOW.py`. I might
open-source that when it is ready, but that is not what this post is
about.

I built Yamark through coding agents. I supplied specifications,
examples, feedback, and product decisions; the retained sessions show
agents writing and committing the code they cover. Git itself records
the commits under my identity, so Git metadata alone cannot establish
that division of labor.

## V1: build it, use it, drain the queue

I remembered V1 as beginning with a basic prompt, followed by a slowly
growing issue queue. The history says otherwise.

The first commit, on May 6, added a 1,416-line `SPEC.md`. Forty-seven
minutes later, the initial implementation commit added 3,410 lines. The
first retained implementation prompt was simply: “Implement SPEC.md.
Spawn subagents where appropriate to do orthogonal chunks of work.” The
session then continued for hours with active direction from me about
architecture, tests, YAML scalars, and Markdown behavior.

Kata arrived two days later, after substantial development. Its drain
loop landed as the 83rd commit. By then the 122-file repository already
contained about 16,600 lines of Rust, JavaScript, Python, and R across
31 source files.

The loop itself was close to what I remembered. A shell script asked
Kata for one ready issue, started a fresh agent session, and stopped if
that session failed. Its prompt allowed an oversized issue to be
decomposed, but required the agent to work on exactly one ready leaf
issue. For code or behavior changes, it required a failing public-API
test before implementation; all completed work had to be tested and
committed before the issue was closed. The Git history stayed linear on
`main`.

What I did not remember was how much of the queue the agents created.
The first 32 issues were loaded in about 70 minutes, several by
decomposing an existing YAML support document. The local Kata database
records 355 issues from May 8 through May 19 UTC: 351 completed, two
closed with no code change, and two duplicates. There were 256 parent
links and 321 blocking links. Annoyances I encountered in daily use fed
the queue, but this was not a story in which I patiently typed every
issue by hand.

V1 reached 448 commits over twelve days. I enabled it globally and it
worked well enough that I wanted to share it. Then I inspected its
architecture.

Here my memory was particularly unfair. V1 was not “regexes on top of
regexes,” and it was not “not a parser in any sense.” It did not even
depend on Rust's regex crate. It had hand-written YAML scanners and
parsers, a semantic graph, source-slice emission, and a Markdown engine.

The real problem was architectural drift. V1 had become a monolithic
collection of scanners, parsers, semantic passes, and formatting
heuristics. By the end, `fast_yaml.rs` was 10,085 lines,
`fast_markdown.rs` was 5,207, and `formatter.rs` was 4,582. An
agent-written architectural review described the drift as “substantial
but salvageable” and the implementation as multi-pass and
allocation-heavy. It parsed the input; it just no longer had the simple,
source-oriented shape I wanted.

So I started V2.

## V2: review the whole specification, fix what remains

My recollection is that I gave ChatGPT Pro the V1 tests and behaviors
and asked it to turn them into a specification, then spent another long
session describing the architecture I wanted. The local history cannot
verify which chat surface produced the documents or how long that took.
It does show the resulting artifacts.

The new repository was called `yamark2`. It was fresh, but not empty:
its first commit on May 18 already contained a 1,203-line `SPEC.md`, a
60-line `ARCHITECTURE.md`, Rust scaffolding, five case fixtures, and
their test harness—3,521 lines in all. Its README explicitly called the
code an intentionally incomplete first-draft template.

The architecture was the important part. I wanted parsing to produce a
source-backed in-memory model, with unchanged regions represented as
spans into the original buffer. Layout and emission decisions should be
made early enough that final emission is mostly mechanical. That design
minimizes copying; it does not literally eliminate output allocations,
as I sometimes overstate when describing it.

The first recorded prompt was: “Implement the SPEC.md here. Do not stop
until all the features described in SPEC.md are implemented.” Nineteen
minutes of agent work plus a later request to commit produced the first
large implementation patch.

Then I set up a different loop. A review agent compared the entire
repository with the entire specification and returned structured
findings. It was allowed to inspect code, run tests, and improve
public-API tests, but not to fix product code. If it found a gap, a
separate implementation agent received those findings, fixed them,
tested the public behavior, and committed. Then review began again.

The exact prompts appear in retained sessions from May 21. The driver
itself was committed on May 28, after it had already been used. Matching
sessions span May 21 through May 30, with a six-day pause in the middle,
so this was several long bursts rather than one process running
continuously for nine days.

The timing is the part I remembered most clearly—and also too neatly.
Early reviews were short and fixes were long. Later reviews were
generally longer than early reviews, while fixes were generally shorter.
But the raw series is noisy; two smooth lines did not steadily cross.
The rolling review median first exceeded the rolling fix median in the
ten-pair window covering pairs 25–34.

![Review and fix timings](blog-assets/review-fix-timings.png)

| Phase               | Review time | Fix time |
| ------------------- | ----------: | -------: |
| Pairs 1–10, median  |      4m 25s |  10m 33s |
| Pairs 11–20, median |      7m 35s |  10m 01s |
| Pairs 21–30, median |      9m 09s |   9m 30s |
| Pairs 31–40, median |      8m 59s |   9m 56s |
| Pairs 41–50, median |      8m 36s |   6m 09s |
| Pairs 51–60, median |     10m 32s |   7m 39s |
| Pairs 61–70, median |      8m 50s |   5m 15s |
| Last completed pair |      7m 11s |   2m 49s |
| Terminal review     |  6h 52m 20s |        — |

The retained logs contain 164 calls using the review/fix prompt family:
89 reviews and 75 fixes. For the comparison above, I paired a completed
`spec-completed=no` review with a fix that began within five seconds.
That produced 73 review→fix attempts; 71 fixes completed and two were
aborted. The chart marks the aborted fixes but excludes their fix
durations from the fix rolling median. It omits 15 unmatched reviews and
two seed fixes. The unmatched reviews include two earlier
`spec-completed=yes` results and one non-specification diff review. I
added the terminal review separately because it is the endpoint I
remembered.

The chart uses wall time from session start to final event. The terminal
session's own internal duration field reports 55m 36s, so its 6h 52m
wall time includes time not counted by the reported task duration. The
selected, public-safe data used in the chart are in the
[timing CSV](blog-assets/review-fix-timings.csv). Exact timestamps and
session identifiers are omitted; the local source logs remain available
for a separate private provenance manifest.

The last successful fix took 2m 49s and canonicalized Markdown table
cells. The review that followed returned no remaining product-code gaps.
Its wall time was not merely a little over two hours, as I remembered,
but nearly seven hours: 6h 52m.

## “Spec complete” was a checkpoint

The review loop worked, but it did not prove that the original
specification captured what I wanted. In fact, it was never quite a
fixed specification. Two earlier invocations had already returned
`spec-completed=yes`; later changes reopened the loop. Each “yes” was a
statement about one repository snapshot, not the end of product
discovery.

The initial document described itself as “current behavior and inferred
product intent” and said it should be refined before being treated as
final. It was edited 18 times, growing from 1,203 to 1,355 lines, before
being deleted. The code and the spec were teaching each other.

Some concrete examples:

- The initial scalar policy favored preserving a quoted YAML string's
  spelling when it was safe. On May 29 I changed the specification to
  prefer an unquoted spelling whenever that was safe, then changed the
  formatter to match.

- The terminal review declared the spec complete on May 30. Less than
  five hours later, a new spec patch added layout intent that had been
  missing: unmatched `[` or `{` could request flow-style layout, while
  physical newlines inside flow collections could request expanded
  layout. The implementation followed 36 minutes later.

- On June 1 the spec was updated to include public `dump-ast` and
  `emit-ast` commands. The next day I decided they did not belong and
  removed the feature, deleting 3,083 lines. This was not an implementer
  accidentally missing a requirement. The feature had matched the spec,
  then I removed it without updating the now-stale spec; that section
  remained wrong until `SPEC.md` itself was deleted.

- The initial spec required wide Markdown tables to fall back to a
  compact form beyond a width threshold. After the spec was gone, I
  found that I preferred aligned tables even when they were wide and
  removed the fallback.

- The initial spec presented the Git filter as supported. Its eventual
  documentation labels it experimental, which better reflects its
  operational tradeoffs.

There are earlier examples too. V1's first spec required an
unchanged-file cache; within 24 minutes I had removed it from the active
spec and moved its design to future work: “We reparse every time.” That
spec initially preserved Markdown bodies byte-for-byte and excluded
fenced-code formatting. Both decisions changed almost immediately. It
treated duplicate YAML keys as a validation error until a Kata issue
reframed Yamark as a formatter, not a schema validator.

This is the part I do not think a longer planning session would have
solved. Knowing what I wanted was coupled to using an implementation and
watching it be implemented. The working formatter produced examples; the
examples changed the desired behavior; the changed behavior exposed
architecture that either helped or got in the way.

That is close to the old agile observation that useful feedback starts
when working software exists. The difference here is the cost structure.
Once V2 had a coherent parser, source model, and test surface, many
later discoveries became small patches instead of architectural
emergencies.

The V2 main branch contains 593 commits. `SPEC.md` was deleted on June
6, and another 36 commits followed before that repository's final commit
on June 15. The public repository was then created with a fresh root
commit containing the then-current tree. Since then, ordinary use has
produced small, concrete patches: spaced image attributes, multiline raw
HTML, heading whitespace, repeated spaces in wrapped YAML scalars, and
multiline brace attributes. Those are exactly the sorts of preferences
and edge cases I could not have specified convincingly in advance.

I now inspect those patches and ask the agent to add and run public-API
regression tests, but I still commit directly to `main`; there is no
conventional pull-request or second-reviewer process for these tweaks. I
have not encountered a significant regression in daily use, though the
history cannot prove the absence of regressions.

## Why Rust

Of course this is written in Rust. I knew Rust in the before times, so
it was comfortable enough. More importantly, I like it for agent-written
systems code. The compiler gives strong, fast, consistent feedback about
reality. An agent can be wrong about ownership, lifetimes, or
representation; `rustc` does not negotiate. A strict compiler is an
LLM's best friend.

## What I took from it

The first loop was product discovery through use: keep a working
formatter nearby, turn annoyances into issues, and drain them one at a
time. The second loop was convergence against a broad contract:
repeatedly separate whole-spec review from implementation. Neither
removed the need for judgment. Together they have so far produced a
codebase whose behavior can keep changing without requiring another
rewrite.

Playing with LLM-assisted workflows is fun because the practice is not
settled. It feels a little like playing with gasoline as a child:
dangerous, fascinating, and clearly not the final machine. The
internal-combustion-engine phase, with all its useful and
society-changing machinery, may come. For now there is still awe and joy
in being close to the raw power and novelty. It smells a little, and it
is probably bad to breathe for too long. The analogy has more mileage
than I first thought.

## History note

I checked the V1, V2, and current Git histories; the retained agent
sessions; the V1 Kata database; and the checked-in benchmark artifacts.
The main history anchors are:

| Event                                   | Commit    |
| --------------------------------------- | --------- |
| V1 initial spec                         | `0af138c` |
| V1 initial implementation               | `eef874b` |
| V1 drain loop                           | `fdc9c62` |
| V1 development endpoint                 | `c8096c2` |
| V2 initial spec and scaffold            | `967af60` |
| V2 first large implementation           | `3feacdd` |
| V2 review/fix driver committed          | `b57b4ca` |
| Final fix before successful spec review | `1caeba3` |
| V2 spec deleted                         | `3b1f48e` |
| V2 development endpoint                 | `6cba8f3` |
| Current public history root             | `609b4a3` |

The current public repository does not contain V1 or V2 ancestry, so
these hashes will become useful links only after I publish a reviewed
history archive. Raw agent sessions are not ready for wholesale
publication: they contain absolute paths, system instructions,
permissions, and unrelated context. The timing CSV is derived from exact
historical prompt matches but contains only derived indices, elapsed
times, roles, and completion states. Exact timestamps, session
identifiers, and source-file hashes are omitted from the public data and
remain available in the local source logs.

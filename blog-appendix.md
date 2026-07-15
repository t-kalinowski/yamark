# Yamark: evidence and methodology

This appendix records the evidence behind
[the Yamark post](blogpost.md). It separates repository facts from
personal recollection and documents how the review/fix timing chart was
constructed.

## Repository reconstruction

I inspected three distinct histories:

- V1, whose selected development endpoint is `c8096c2`;
- V2, whose endpoint is `6cba8f3`;
- the current public repository, which begins at a fresh root,
  `609b4a3`, and does not preserve V1 or V2 ancestry.

V1 began on May 6, 2026. Its first commit added a 1,416-line `SPEC.md`;
the initial implementation followed 47 minutes later with 3,410 inserted
lines. The drain loop landed as the 83rd commit. V1 reached 448 commits
over twelve days.

Before the drain-loop commit, the 122-file repository contained 16,656
lines of Rust, JavaScript, Python, and R across 31 source files. At the
V1 endpoint, the largest modules were `fast_yaml.rs` at 10,085 lines,
`fast_markdown.rs` at 5,207, and `formatter.rs` at 4,582.

V2 began on May 18 with a 1,203-line `SPEC.md`, a 60-line
`ARCHITECTURE.md`, Rust scaffolding, five case fixtures, and their test
harness. Its `main` branch contains 593 commits. `SPEC.md` was deleted
on June 6; 36 more commits followed before the V2 endpoint on June 15.

## Kata issue-store reconstruction

Kata was introduced after substantial V1 development. The first 32
issues were loaded in about 70 minutes, several by decomposing an
existing YAML support document.

The local Kata database records 355 Yamark issues between May 8 and May
19 UTC:

- 351 completed;
- two closed after an audit found no code change was needed;
- two duplicates;
- 256 parent links;
- 321 blocking links.

The drain script selected one ready issue and started a fresh agent
session. Oversized issues could be decomposed, but a session then worked
on one ready leaf. For code or behavior changes, the prompt required a
failing public-API test before implementation. Completed work had to be
tested and committed before the issue closed.

## Session matching and timing definitions

The retained logs contain 164 calls in the review/fix prompt family: 89
reviews and 75 fixes. The plotted subset pairs a completed
`spec-completed=no` review with a fix beginning within five seconds. It
contains 73 review→fix attempts: 71 completed fixes and two aborted
fixes. The terminal `spec-completed=yes` review is added separately
because it is the endpoint discussed in the article.

The subset omits 15 unmatched reviews and two seed fixes. The unmatched
reviews include five aborted reviews, two earlier `spec-completed=yes`
results, reviews without an immediately following fix, and one
non-specification diff review. Aborted fix durations remain visible in
the chart but are excluded from the fix rolling median; their
corresponding review durations remain included.

Durations are wall time from the session timestamp to its final event.
The terminal review remained open for 24,740 seconds, or 6h 52m 20s. Its
reported task-duration field is 3,335.743 seconds, or 55m 36s, so wall
time includes time outside the reported task duration.

| Paired attempts | Median review | Median fix |
| --------------- | ------------: | ---------: |
| 1–10            |        4m 25s |    10m 33s |
| 11–20           |        7m 35s |    10m 01s |
| 21–30           |        9m 09s |     9m 30s |
| 31–40           |        8m 59s |     9m 56s |
| 41–50           |        8m 36s |     6m 09s |
| 51–60           |       10m 32s |     7m 39s |
| 61–70           |        8m 50s |     5m 15s |

The rolling review median first exceeded the rolling fix median in the
ten-pair window covering pairs 25–34. The raw series is noisy; the
crossing is a change in central tendency, not a monotonic progression.

The [public timing CSV](blog-assets/review-fix-timings.csv) contains
derived indices, elapsed times, roles, and completion states. The
[plotting script](blog-assets/plot-review-fix-timings.py) produces the
chart in the article. Exact timestamps, session identifiers, absolute
paths, and source-file hashes are intentionally absent from the public
data.

## Specification changes

The V2 specification was edited 18 times. It grew from 1,203 lines and
45,250 bytes to 1,355 lines and 59,331 bytes before deletion. The main
article describes four representative changes.

V1 had similar early changes:

- its required unchanged-file cache moved out of the active
  specification and into future work within 24 minutes;
- its initial byte-preserving Markdown policy and exclusion of
  fenced-code formatting were replaced by active Markdown and fence
  formatting;
- duplicate YAML keys changed from a validation error to accepted
  formatter input.

## Commit anchors

| Event                                 | Commit    |
| ------------------------------------- | --------- |
| V1 initial specification              | `0af138c` |
| V1 initial implementation             | `eef874b` |
| V1 drain loop                         | `fdc9c62` |
| V1 development endpoint               | `c8096c2` |
| V2 initial specification and scaffold | `967af60` |
| V2 first large implementation         | `3feacdd` |
| V2 review/fix driver committed        | `b57b4ca` |
| Final fix before the terminal review  | `1caeba3` |
| V2 specification deleted              | `3b1f48e` |
| V2 development endpoint               | `6cba8f3` |
| Current public-history root           | `609b4a3` |

These hashes will become public links after a reviewed evidence archive
is published.

## Publication and provenance boundaries

Raw agent sessions are not suitable for wholesale publication. They
contain absolute paths, system and developer instructions, permission
details, and unrelated context. The intended public evidence package is
narrower: faithful Git histories, exact workflow prompts, driver
scripts, sanitized timing data, plotting code, a commit-to-event index,
matching methodology, and hashes for private source records.

A private provenance manifest should retain the exact session
identifiers, UTC intervals, source-file hashes, repository states,
structured results, and resulting commits. That preserves local
reproducibility without exposing the raw conversation bodies.

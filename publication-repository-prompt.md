# Prompt: prepare the Yamark history archive

This is a private operational handoff. It contains local paths and
private provenance. Do not commit or publish this prompt; use it to
produce the sanitized reader-facing archive described below.

Prepare a new local Git repository that lets readers inspect Yamark V1,
V2, and the current public release history. Do not publish or push
anything until I have reviewed the privacy, licensing, and contents
report.

Source repositories:

- V1: `/Users/tomasz/github/t-kalinowski/yamark-proto/yamark-v1`
- V2: `/Users/tomasz/github/t-kalinowski/yamark-proto/yamark2`
- Current public repository: `/Users/tomasz/github/t-kalinowski/yamark`
- V1 Kata database: `/Users/tomasz/.kata/kata.db`, project ID 2 only
- Derived timing data:
  `/Users/tomasz/github/t-kalinowski/yamark/blog-assets/review-fix-timings.csv`

Use only `git` and `gh` for GitHub work. Do not modify the three source
repositories. Preserve historical commit and tree hashes; do not rewrite
V1 and V2 into a synthetic linear history. Build one archive repository
with a non-historical default branch and unrelated archival branches
such as `history/v1`, `history/v2`, and `history/current`.

Preserve and document these anchors:

- V1 initial spec: `0af138c4057d2f36730de81dba681890f4e3c34f`
- V1 initial implementation: `eef874b72aaf2590171a9cddd2ea944fa68107ee`
- V1 drain loop: `fdc9c6210815c08726d2c55e0f4d08c47c987dce`
- V1 selected development endpoint:
  `c8096c24e11a3064ff65ec028a13954ac6807c0d`
- V1 later local maintenance commit:
  `1487b1cdfe908462c64725fdbd555b521c75e9c6`
- V2 initial spec and scaffold:
  `967af60352cf6822a583c327bed412eba1681b4a`
- V2 first large implementation:
  `3feacdd94d5dd348659da534cb04f5d870eae030`
- V2 review/fix driver committed:
  `b57b4ca3ad03c39babe6d39030973ad5d3185b87`
- Final fix before successful spec review:
  `1caeba304a4f4a31f2ada5a81c7cef9a8494103a`
- V2 spec deleted: `3b1f48edafae3b19604c5b5efadf013625623e0a`
- V2 development endpoint: `6cba8f362bbe76768e8f3062ef7c311588bb0e20`
- Current public history root:
  `609b4a39fb4bc4b6c507419ee19eeea19fd36a13`

Verify every hash before doing other work and fail if any anchor is
missing. Create annotated tags for the important milestones. Treat
`1caeba3` as the code state reviewed by the terminal session; the review
itself is session evidence, not a Git commit.

Record the terminal review in the private provenance manifest: session
`019e76dd-d37a-77b1-8d81-1984b2b05fdf`, wall interval
`2026-05-30T03:11:49.653Z` – `2026-05-30T10:04:09.692Z`, reported task
duration 3,335.743 seconds, result `spec-completed=yes`, and source-file
SHA-256
`a641889847fb7cd8cff1302be163384174d94741123564a907d839fcd989d083`.
Verify these values from the source file before relying on them.

The current repository's reflog and unreachable objects contain
pre-squash release-preparation history. Inspect them and import
meaningful chains into explicit archive refs without relying on reflogs
or unreachable objects in the finished archive. Do not change
source-repository refs. Document which chains were retained or omitted
and why.

On the documentation branch, add:

- a concise README with the chronology, branch/tag map, and exact
  `git worktree add` commands for inspecting each version;
- `evidence/commit-map.csv` with event, repository, full hash, author
  time, commit time, and subject;
- the source and rendered form of the V1 issue-drain versus V2
  review/fix workflow diagram used in the article;
- the public-safe review/fix timing CSV and plot;
- the extraction script, exact matching/pairing rules, and explicit
  inclusion/exclusion reasons;
- a sanitized inventory of all 164 matching calls, not only the 147
  calls used in the plotted subset, so readers can audit the selection;
- a note that readers cannot fully reproduce extraction without the
  private raw sessions, plus hashes for the private inputs so I can
  reproduce it locally;
- a sanitized export of Kata project ID 2 only, preserving issue IDs,
  titles, status, timestamps, parent links, and blocking links;
- a single verification command that checks all recorded hashes and tree
  hashes, verifies branch/tag targets, and builds/tests the historical
  endpoints where their checked-in toolchains permit it;
- a limitations document listing evidence that is missing, derived,
  omitted, or based on personal recollection.

Do not publish the full Kata database or raw `~/.codex/sessions` trees.
Raw sessions contain absolute paths, system/developer instructions,
permissions, and possibly unrelated context. First produce an inventory
and privacy report. If selected transcripts would materially help
readers, propose a minimal reviewed subset and a deterministic
sanitization scheme, but do not add transcript bodies without explicit
approval. Keep exact session UUIDs, exact UTC timestamps, local paths,
and source-file hashes in a private manifest outside the public archive.
Use derived indices, elapsed durations, roles, statuses, and documented
exclusion reasons in the public data.

Audit every branch, tag, and added artifact for secrets, tokens,
credentials, private paths, email/PII, unrelated prompt content, large
generated files, and third-party material. Also audit licensing:
historical V1/V2 manifests declare MIT, but their tips may lack a root
license file. Add archive-level licensing and provenance on the
documentation branch without changing historical trees, and report any
unresolved licensing question.

Treat expected Git author names and email addresses as historical
provenance, while still reporting them for review; distinguish them from
unexpected private PII in content or metadata. If any historical ref
fails the safety or licensing audit, do not rewrite it to make it
publishable. Withhold that ref and report the exact blocker.

Keep archive branches byte-for-byte faithful. Put explanations,
sanitized evidence, and verification tooling only on the documentation
branch. Make the reader workflow one clear path: clone, inspect the
README, create a worktree for a named historical branch, and run its
documented checks.

Before stopping, show:

1. the local archive path;
2. all branches and annotated tags with full target hashes;
3. verification results and any historical check that cannot run;
4. the privacy, secret, and license audit results;
5. a full summary of documentation-branch changes;
6. the exact proposed `gh repo create` and `git push` commands, without
   executing them.

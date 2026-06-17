---
title: Git Filter
description: Store Markdown sentence-per-line while editing column-wrapped files.
---

Column-wrapped Markdown is comfortable in an editor, but a one-sentence edit in
the middle of a paragraph can rewrap the whole paragraph. Yamark's Git filter
stores sentence-per-line Markdown in the index and writes column-wrapped
Markdown into the working tree.

The filter only applies to paths matched by Git attributes.

## Experimental status

The Git filter is experimental and may change or be removed. The design idea is
to normalize Markdown at the Git boundary: agents get a sentence-per-line form
that is easier to diff and edit mechanically, while humans get column-wrapped
files in the editor.

That boundary is imperfect. On checkout, Git writes the smudged working-tree
view, so the working tree can still be rewritten into the human-facing format
before an agent sees it. Treat this feature as a design experiment, not a stable
repository policy.

## Behavior

`clean` runs when content moves from the working tree into the index:

```sh
yamark git-filter clean --stdin-filename docs/file.md
```

It formats Markdown with sentence wrapping.

`smudge` runs when content moves from the index into the working tree:

```sh
yamark git-filter smudge --stdin-filename docs/file.md --markdown-wrap-at-column 72
```

It formats Markdown with column wrapping. Both directions use compact pipe-table
output to keep storage diffs small.

Unsupported paths fail. The filter is for Markdown-like files, not YAML or
source files.

## Adopt a repository

For a shared repository, the maintainer path is:

```sh
git switch -c adopt-yamark-filter
yamark git-filter adopt
git diff --cached
git commit -m "Normalize Markdown with yamark Git filter"
```

`adopt` requires a clean worktree. It writes shared `filter=yamark-md`
patterns to `.gitattributes`, installs the local filter driver, normalizes
tracked Markdown blobs into sentence-per-line storage, stages the normalized
files, and checks the codec invariant:

```text
clean(smudge(blob)) == blob
```

Use `--markdown-wrap-at-column <n>` to change the working-tree wrap width. Pass
paths to adopt only specific Markdown files:

```sh
yamark git-filter adopt content/blog/post/index.md
```

## Join a repository

Contributors should join only after the adoption commit is present in their
checkout:

```sh
git pull --ff-only
yamark git-filter join
```

`join` refuses when the current branch is behind its upstream. After the branch
is current, it installs the local filter driver, verifies committed blobs
round-trip, writes the column-wrapped working-tree view, and leaves
`git status --porcelain` clean.

## Check in CI

Use `check` in CI to verify committed blobs still satisfy the clean/smudge
contract:

```sh
yamark git-filter check
```

It fails if any tracked Markdown file selected by `.gitattributes` violates:

```text
clean(smudge(blob)) == blob
```

## Local setup

For personal experiments in the current repository, run:

```sh
yamark git-filter setup
```

This low-level command configures the filter driver and adds the default
Markdown patterns to `.git/info/attributes`. It does not create a shared
normalization commit. Pass paths to configure only those paths:

```sh
yamark git-filter setup content/blog/post/index.md
```

Use `--repo path/to/repo` when running from outside the repository. Use
`--markdown-wrap-at-column <n>` to change the working-tree wrap width.

Setup prints the local files and config keys it changed, plus the commands below
for undoing or bypassing the filter.

To configure the same pieces manually, set the filter driver:

```sh
git config filter.yamark-md.clean "yamark git-filter clean --stdin-filename %f"
git config filter.yamark-md.smudge "yamark git-filter smudge --stdin-filename %f --markdown-wrap-at-column 72"
git config filter.yamark-md.required true
git config merge.renormalize true
```

For personal use, add attributes to `.git/info/attributes`. For shared project
storage, use `yamark git-filter adopt` so the attributes and normalized blobs
land in the same commit. The shared attributes look like:

```gitattributes
*.md filter=yamark-md
*.qmd filter=yamark-md
*.Rmd filter=yamark-md
*.rmd filter=yamark-md
```

If you use low-level local setup on existing tracked files, normalize the index
explicitly:

```sh
git add --renormalize .
```

Review the staged diff before committing the renormalization.

## Undo

To remove the local setup created by `yamark git-filter setup`, run:

```sh
yamark git-filter teardown
```

This unsets the local filter config and removes simple
`filter=yamark-md` lines from `.git/info/attributes`. If your repository has
committed `filter=yamark-md` rules in `.gitattributes`, edit that file directly.

To stage one file exactly as it appears in the working tree without running the
clean filter for that `git add`, use:

```sh
git -c filter.yamark-md.clean=cat add NEWS.md
```

## Per-file control

Git attributes choose which paths use the filter. Put more specific rules after
broad ones:

```gitattributes
*.md filter=yamark-md
NEWS.md -filter
```

Use this when a file should stay out of the clean/smudge flow. For example, if
`NEWS.md` should stay column-wrapped in both the working tree and the index,
exclude it from the filter and use the normal formatter:

```sh
yamark format --wrap 72 NEWS.md
git add NEWS.md
```

Use `<!-- fmt: skip file -->` inside a Markdown file when the file should still
pass through the filter but Yamark should preserve its contents.

## Diff shape

Without the filter, editing one sentence inside a wrapped paragraph can modify
every physical line in the paragraph.

```diff
-Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do
-eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut
-enim ad minim veniam, quis nostrud exercitation ullamco laboris.
+Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do
+eiusmod tempor incididunt ut labore et dolore magna aliqua. The
+second sentence is the one that actually changed in this commit.
```

With the filter, the stored form is sentence-per-line, so the changed sentence
is the changed line.

```diff
-Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.
+The second sentence is the one that actually changed in this commit.
```

## Notes

Every clone that wants the working-tree view needs the `git config` commands.
The `.gitattributes` file selects paths, but it does not define the filter
driver by itself.

Use a pre-commit hook or CI `yamark format --check` instead when you want the
repository storage form to match the working-tree form.

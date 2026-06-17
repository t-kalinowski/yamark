# Markdown Support Gap Workpad

This workpad turns trace findings from the book fixture corpus into an
implementation checklist. Use it as a progress document: add a public CLI or
public API regression test first, confirm it fails, implement one feature, then
mark the item done only after the test passes and the diagnostics improve.

Baseline command used to identify these gaps:

```sh
cargo run --features format-trace -- format --diagnostics .
```

Run it from a fixture corpus such as
`/private/tmp/yamark-book-fixtures-20260601-061430`. The June 1, 2026 run found
14,240 skipped Markdown nodes and 97 optional embedded formatter chunk failures.
The largest skipped buckets were raw blocks, code fences, paragraphs, Quarto
divs, HTML comments, lists, and blockquotes.

Follow-up diagnostics on the same fixture corpus after the checklist work found
8,247 skipped Markdown nodes and 130 optional embedded formatter chunk failures.
The run scanned 22,625 files, formatted 602, left 1,531 unchanged, skipped
20,492 unsupported files, and had 0 hard failures. Skipped Markdown nodes by kind:
Raw 3,908, CodeFence 2,151, HtmlComment 819, Paragraph 810, List 551, QuartoDiv
5, Heading 2, and Blockquote 1. Optional embedded formatter failures were ruff
113, air 9, and prettier 8.

## Working Rules

- Test only public behavior: CLI cases, public format functions, or external
  smoke tests.
- Add the failing test first and keep each test focused on one syntax rule.
- Preserve source when syntax is semantically opaque, but do not let one opaque
  child block force a larger parent node to be copied if the parent structure is
  otherwise understandable.
- Treat template and shortcode spans as protected tokens. Do not rewrite their
  internals.
- Prefer one happy path per syntax feature. Fail fast on malformed or ambiguous
  constructs that the public parser cannot safely model.
- Re-run a focused diagnostics sample after each group and compare skipped node
  counts by kind.

## Progress Checklist

- [x] Add a diagnostics fixture that asserts the skipped count drops for each
      supported feature group.
- [x] Support Quarto/Pandoc fenced divs.
- [x] Support Quarto executable code fences with braced info strings.
- [x] Support Pandoc code fence attributes.
- [x] Support MyST/JupyterBook `code-cell` fences.
- [x] Support raw-format code fences.
- [x] Support tables containing template or shortcode spans.
- [x] Support multiline links and images.
- [x] Support link labels with inline markup.
- [x] Support inline HTML tokens in prose.
- [x] Support GFM strikethrough spans.
- [x] Support lists with rich child blocks.
- [x] Support blockquotes with rich child blocks.
- [x] Support Pandoc footnotes with rich inline content.
- [x] Support Quarto/Pandoc shortcodes and includes as block syntax.
- [x] Support display math blocks.
- [x] Re-run corpus diagnostics and update this document with before/after
      counts.

## 1. Quarto And Pandoc Fenced Divs

Expected behavior:

Recognize fenced div openers using three or more colons and matching closers of
at least the same length. Preserve attributes exactly unless the existing
attribute normalizer already has a safe rule for them. Format supported Markdown
inside the div recursively. Keep the div fence itself in place, keep one blank
line around block children where normal Markdown formatting would require it,
and never copy the entire div only because a child paragraph can be wrapped.

Implementation checklist:

- [x] Parse div opener, attributes, body span, and matching closer.
- [x] Support nested divs with longer or equal fence lengths.
- [x] Format child Markdown recursively with the same directive state.
- [x] Preserve opaque children, but continue formatting siblings.

Example 1:

Before:

~~~~~md
::: {.callout-tip}
## Learn more
See the [guide](https://quarto.org/docs/guide.html).
:::
~~~~~

After:

~~~~~md
::: {.callout-tip}
## Learn more

See the [guide](https://quarto.org/docs/guide.html).
:::
~~~~~

Example 2:

Before:

~~~~~md
:::: columns
::: {.column width="50%"}
- one
- two
:::
::: {.column width="50%"}
Text in the second column.
:::
::::
~~~~~

After:

~~~~~md
:::: columns
::: {.column width="50%"}
- one
- two
:::

::: {.column width="50%"}
Text in the second column.
:::
::::
~~~~~

Example 3:

Before:

~~~~~md
::: {.panel-tabset}
## R
```{r}
plot(cars)
```
## Python
```{python}
print("cars")
```
:::
~~~~~

After:

~~~~~md
::: {.panel-tabset}
## R

```{r}
plot(cars)
```

## Python

```{python}
print("cars")
```
:::
~~~~~

## 2. Quarto Executable Code Fences

Expected behavior:

Recognize braced executable fence info such as `{python}`, `{r}`, `{ojs}`, and
`{bash}`. Infer the language from the first token inside braces. Preserve chunk
options and comments. Format the code body with the existing embedded formatter
when the language maps to a configured formatter and the chunk body is valid for
that formatter. If the external formatter reports an optional failure, preserve
the body and emit the existing diagnostic without marking the whole Markdown
node unsupported.

Implementation checklist:

- [x] Extract language from `{language}` and `{language option=value}`.
- [x] Preserve all chunk options in opener order.
- [x] Keep optional formatter failures local to the chunk body.
- [x] Add cases for `r`, `python`, and `ojs`.

Example 1:

Before:

~~~~~md
```{python}
from pathlib import Path
print( Path("data") )
```
~~~~~

After:

~~~~~md
```{python}
from pathlib import Path

print(Path("data"))
```
~~~~~

Example 2:

Before:

~~~~~md
```{r}
cars |>    plot()
```
~~~~~

After:

~~~~~md
```{r}
cars |> plot()
```
~~~~~

Example 3:

Before:

~~~~~md
```{python echo=false}
x=1
print(x)
```
~~~~~

After:

~~~~~md
```{python echo=false}
x = 1
print(x)
```
~~~~~

## 3. Pandoc Code Fence Attributes

Expected behavior:

Recognize Pandoc attribute syntax after a code fence, including language classes,
identifiers, key-value pairs, and filename attributes. Infer the formatter
language from the first language class or bare language token. Normalize only
safe spacing around the opener; do not reorder attributes or change quoted
attribute values. Preserve attributes even when the code body is passed to an
external formatter.

Implementation checklist:

- [x] Support `{.python #id key="value"}` and bare `python key=value` forms.
- [x] Infer language from `.python`, `python`, `.bash`, `.r`, and `.json`.
- [x] Do not format attributes with quotes or escapes unless already supported.
- [x] Preserve fence length and marker type.

Example 1:

Before:

~~~~~md
``` {.bash filename="Terminal"}
quarto render
```
~~~~~

After:

~~~~~md
```{.bash filename="Terminal"}
quarto render
```
~~~~~

Example 2:

Before:

~~~~~md
``` {#lst-pandas .python lst-cap="Pandas"}
import pandas as pd
df=pd.DataFrame()
```
~~~~~

After:

~~~~~md
```{#lst-pandas .python lst-cap="Pandas"}
import pandas as pd

df = pd.DataFrame()
```
~~~~~

Example 3:

Before:

~~~~~md
~~~ {.json filename="config.json"}
{"a":1,"b":2}
~~~
~~~~~

After:

~~~~~md
~~~{.json filename="config.json"}
{
  "a": 1,
  "b": 2
}
~~~
~~~~~

## 4. MyST And JupyterBook Code-Cell Fences

Expected behavior:

Recognize `code-cell` fences as structured executable cells. Preserve cell
metadata lines that start with `:` before the code body. If the opener includes
an explicit language, use that language for embedded formatting. If no language
is present, preserve the body and still treat the fence as a supported code-cell
node so it does not inflate skipped code fence counts.

Implementation checklist:

- [x] Parse `{code-cell}` and `{code-cell} language` openers.
- [x] Preserve leading colon metadata lines.
- [x] Format only the code body after metadata when a language is known.
- [x] Keep cell metadata attached to the fence.

Example 1:

Before:

~~~~~md
```{code-cell} python
:tags: [hide-input]
x=1
print(x)
```
~~~~~

After:

~~~~~md
```{code-cell} python
:tags: [hide-input]
x = 1
print(x)
```
~~~~~

Example 2:

Before:

~~~~~md
```{code-cell}
:tags: [remove-output]
import numpy as np
```
~~~~~

After:

~~~~~md
```{code-cell}
:tags: [remove-output]
import numpy as np
```
~~~~~

Example 3:

Before:

~~~~~md
```{code-cell} ipython3
value=42
value
```
~~~~~

After:

~~~~~md
```{code-cell} ipython3
value = 42
value
```
~~~~~

## 5. Raw-Format Code Fences

Expected behavior:

Recognize raw-format fences such as `{=html}`, `{=latex}`, and `{=typst}` as
supported opaque code fences. Preserve the body byte-for-byte. Do not send these
chunks to external formatters. The opener and closer should be retained exactly
except for safe code fence opener spacing normalization.

Implementation checklist:

- [x] Parse `{=format}` as raw output, not as a formatter language.
- [x] Preserve body and fence marker length.
- [x] Keep raw fences supported inside lists, blockquotes, and divs.

Example 1:

Before:

~~~~~md
```{=html}
<div class="note">Raw HTML output</div>
```
~~~~~

After:

~~~~~md
```{=html}
<div class="note">Raw HTML output</div>
```
~~~~~

Example 2:

Before:

~~~~~md
```{=latex}
\newpage
```
~~~~~

After:

~~~~~md
```{=latex}
\newpage
```
~~~~~

Example 3:

Before:

~~~~~md
``` {=typst}
#pagebreak()
```
~~~~~

After:

~~~~~md
```{=typst}
#pagebreak()
```
~~~~~

## 6. Tables With Template Or Shortcode Spans

Expected behavior:

Format GFM and Pandoc pipe tables even when a cell contains a template,
shortcode, or mustache-like span. Treat those spans as protected inline tokens
for width calculation and alignment. Do not rewrite shortcode contents. Preserve
escaped pipes and inline code spans when computing cell boundaries.

Implementation checklist:

- [x] Allow protected template tokens in table cells.
- [x] Align columns without changing protected token internals.
- [x] Preserve long cells that exceed line width instead of wrapping inside a
      table cell.

Example 1:

Before:

~~~~~md
| Function | Description |
|---|---|
| `quarto.metadata.get(key)` | Equivalent to `{{< meta key >}}`. |
~~~~~

After:

~~~~~md
| Function                   | Description                              |
| -------------------------- | ---------------------------------------- |
| `quarto.metadata.get(key)` | Equivalent to `{{< meta key >}}`.        |
~~~~~

Example 2:

Before:

~~~~~md
| Name | Value |
|---|---|
| title | `{{{< meta title >}}}` |
~~~~~

After:

~~~~~md
| Name  | Value                    |
| ----- | ------------------------ |
| title | `{{{< meta title >}}}`   |
~~~~~

Example 3:

Before:

~~~~~md
| Include | Target |
|---|---|
| footer | {{< include _footer.md >}} |
~~~~~

After:

~~~~~md
| Include | Target                    |
| ------- | ------------------------- |
| footer  | {{< include _footer.md >}} |
~~~~~

## 7. Multiline Links And Images

Expected behavior:

Recognize links and images whose label or destination spans multiple lines.
Preserve the split destination when it is already split intentionally. Normalize
surrounding whitespace and wrap the containing paragraph around the protected
link token. Do not collapse a multiline destination into one line unless a
specific formatting policy is added and tested.

Implementation checklist:

- [x] Parse multiline inline links and images as protected inline tokens.
- [x] Preserve target indentation and line breaks.
- [x] Support nested image links used for badges.

Example 1:

Before:

~~~~~md
[documentation](
  https://example.com/docs/very/long/path
)
~~~~~

After:

~~~~~md
[documentation](
  https://example.com/docs/very/long/path
)
~~~~~

Example 2:

Before:

~~~~~md
[![status](
  https://img.shields.io/badge/status-ok-green.svg
)](https://example.com/status)
~~~~~

After:

~~~~~md
[![status](
  https://img.shields.io/badge/status-ok-green.svg
)](https://example.com/status)
~~~~~

Example 3:

Before:

~~~~~md
Read the [long guide](
  https://example.com/guide
) before continuing with the tutorial.
~~~~~

After:

~~~~~md
Read the [long guide](
  https://example.com/guide
) before continuing with the tutorial.
~~~~~

## 8. Link Labels With Inline Markup

Expected behavior:

Support inline markup inside link and image labels, including strong emphasis,
emphasis, inline code, math, and nested images when already supported. Preserve
the label markup exactly unless canonical inline formatting is explicitly
enabled. Normalize only whitespace around the overall link token and the
destination formatting rules already supported for simple links.

Implementation checklist:

- [x] Allow emphasis spans inside link labels.
- [x] Allow inline code and math inside link labels.
- [x] Keep unsupported nested labels as explicit errors only when a public input
      can produce ambiguity.

Example 1:

Before:

~~~~~md
See [**Beginner-Friendly Examples**](./examples/README.md).
~~~~~

After:

~~~~~md
See [**Beginner-Friendly Examples**](./examples/README.md).
~~~~~

Example 2:

Before:

~~~~~md
Read [`format.POSIXct()`](https://rdrr.io/r/base/strptime.html).
~~~~~

After:

~~~~~md
Read [`format.POSIXct()`](https://rdrr.io/r/base/strptime.html).
~~~~~

Example 3:

Before:

~~~~~md
See [the *emphasis* section](#emphasis).
~~~~~

After:

~~~~~md
See [the *emphasis* section](#emphasis).
~~~~~

## 9. Inline HTML In Prose

Expected behavior:

Treat inline HTML tags as protected inline tokens when they occur inside
otherwise normal paragraphs, lists, tables, footnotes, and blockquotes. Preserve
the tag text and attributes exactly. Wrap surrounding text without splitting a
tag token. Block-level HTML remains a separate raw block feature and should not
be confused with inline HTML.

Implementation checklist:

- [x] Distinguish inline tags from block HTML.
- [x] Tokenize paired and self-closing inline tags as protected spans.
- [x] Wrap surrounding prose while preserving tag bytes.

Example 1:

Before:

~~~~~md
Press <kbd>Cmd</kbd> + <kbd>K</kbd> to open the launcher.
~~~~~

After:

~~~~~md
Press <kbd>Cmd</kbd> + <kbd>K</kbd> to open the launcher.
~~~~~

Example 2:

Before:

~~~~~md
The value can be <span class="pkg">dplyr</span> in examples.
~~~~~

After:

~~~~~md
The value can be <span class="pkg">dplyr</span> in examples.
~~~~~

Example 3:

Before:

~~~~~md
Use the <br> tag only in generated HTML examples.
~~~~~

After:

~~~~~md
Use the <br> tag only in generated HTML examples.
~~~~~

## 10. GFM Strikethrough

Expected behavior:

Support `~~text~~` as a protected inline span in paragraphs, list items,
blockquotes, footnotes, and table cells. Preserve the delimiter and contents.
Wrap surrounding prose without splitting the strikethrough span. Malformed
unclosed spans should remain unsupported until there is a clear policy.

Implementation checklist:

- [x] Add strikethrough tokenization to inline scanning.
- [x] Preserve escaped tildes.
- [x] Support strikethrough inside links only after link-label markup support is
      in place.

Example 1:

Before:

~~~~~md
Use ~~old option~~ new option instead.
~~~~~

After:

~~~~~md
Use ~~old option~~ new option instead.
~~~~~

Example 2:

Before:

~~~~~md
- ~~Removed item~~
- Kept item
~~~~~

After:

~~~~~md
- ~~Removed item~~
- Kept item
~~~~~

Example 3:

Before:

~~~~~md
| Status | Value |
|---|---|
| old | ~~deprecated~~ |
~~~~~

After:

~~~~~md
| Status | Value          |
| ------ | -------------- |
| old    | ~~deprecated~~ |
~~~~~

## 11. Lists With Rich Child Blocks

Expected behavior:

Format list item text while preserving and recursively formatting supported
child blocks such as fenced code, nested lists, blockquotes, tables, math, and
divs. Keep child block indentation relative to the list marker. Do not copy the
whole list only because one item contains a code fence. Preserve ordered list
marker values unless a canonical renumbering option is explicitly enabled.

Implementation checklist:

- [x] Represent list item children structurally.
- [x] Format paragraph children independently from fenced child blocks.
- [x] Preserve item marker style and numbering by default.
- [x] Support blank lines between loose list children.

Example 1:

Before:

~~~~~md
- Install dependencies:

  ```bash
  npm install
  ```

- Run the app.
~~~~~

After:

~~~~~md
- Install dependencies:

  ```bash
  npm install
  ```

- Run the app.
~~~~~

Example 2:

Before:

~~~~~md
1. Load data.
1. Build a table:

   | a | b |
   |---|---|
   | 1 | 2 |
~~~~~

After:

~~~~~md
1. Load data.
1. Build a table:

   | a   | b   |
   | --- | --- |
   | 1   | 2   |
~~~~~

Example 3:

Before:

~~~~~md
- Tip:
  > Quote inside a list.
- Continue.
~~~~~

After:

~~~~~md
- Tip:
  > Quote inside a list.
- Continue.
~~~~~

## 12. Blockquotes With Rich Child Blocks

Expected behavior:

Parse blockquote contents as nested Markdown instead of a flat wrapped
paragraph. Format supported children recursively and reapply quote prefixes.
Preserve nested blockquote depth. Support tables, lists, fenced code, divs, and
math inside a blockquote without copying the whole quote block.

Implementation checklist:

- [x] Strip quote markers into a nested Markdown source.
- [x] Format nested source.
- [x] Reapply quote markers and preserve blank quoted lines.
- [x] Support nested blockquote levels.

Example 1:

Before:

~~~~~md
> | Term | Meaning |
> |---|---|
> | ML | Machine learning |
~~~~~

After:

~~~~~md
> | Term | Meaning          |
> | ---- | ---------------- |
> | ML   | Machine learning |
~~~~~

Example 2:

Before:

~~~~~md
> ```bash
> quarto preview
> ```
~~~~~

After:

~~~~~md
> ```bash
> quarto preview
> ```
~~~~~

Example 3:

Before:

~~~~~md
> - first item
> - second item
>
> Final sentence.
~~~~~

After:

~~~~~md
> - first item
> - second item
>
> Final sentence.
~~~~~

## 13. Pandoc Footnotes With Rich Inline Content

Expected behavior:

Format footnote definitions as paragraph-like blocks with a stable continuation
indent. Support rich inline content in the footnote body, including inline code,
link labels with inline markup, math, and emphasis. Preserve multiple paragraph
footnotes and indented child blocks. Do not rewrite footnote labels.

Implementation checklist:

- [x] Reuse paragraph inline support in footnote bodies.
- [x] Preserve continuation indentation.
- [x] Support multi-paragraph footnote bodies.
- [x] Support footnote bodies with child code fences only after rich child block
      support is implemented.

Example 1:

Before:

~~~~~md
[^note]: See [`library()` vs `require()`](https://yihui.org/en/2014/07/library-vs-require/).
~~~~~

After:

~~~~~md
[^note]: See [`library()` vs `require()`](https://yihui.org/en/2014/07/library-vs-require/).
~~~~~

Example 2:

Before:

~~~~~md
[^math]: The value is `$x + y$` in the notation used here.
~~~~~

After:

~~~~~md
[^math]: The value is `$x + y$` in the notation used here.
~~~~~

Example 3:

Before:

~~~~~md
[^long]: First paragraph of the footnote.
    Second paragraph with [**strong label**](https://example.com).
~~~~~

After:

~~~~~md
[^long]: First paragraph of the footnote.
    Second paragraph with [**strong label**](https://example.com).
~~~~~

## 14. Quarto And Pandoc Shortcode Blocks

Expected behavior:

Recognize shortcode and include lines as supported opaque block nodes. Preserve
the shortcode line exactly. Do not let a shortcode block cause adjacent
paragraphs, lists, or divs to be copied. Paired shortcodes should preserve their
body unless a later feature explicitly models the body as Markdown.

Implementation checklist:

- [x] Parse single-line shortcode blocks.
- [x] Parse paired shortcode blocks.
- [x] Preserve shortcode internals byte-for-byte.
- [x] Keep surrounding Markdown format decisions independent.

Example 1:

Before:

~~~~~md
{{< include _footer.md >}}
~~~~~

After:

~~~~~md
{{< include _footer.md >}}
~~~~~

Example 2:

Before:

~~~~~md
{{< video https://www.youtube.com/embed/abc123 >}}
~~~~~

After:

~~~~~md
{{< video https://www.youtube.com/embed/abc123 >}}
~~~~~

Example 3:

Before:

~~~~~md
Intro paragraph.

{{< meta title >}}

Closing paragraph.
~~~~~

After:

~~~~~md
Intro paragraph.

{{< meta title >}}

Closing paragraph.
~~~~~

## 15. Display Math Blocks

Expected behavior:

Recognize display math delimited by `$$`, `\[...\]`, and common LaTeX
environment blocks. Preserve math body byte-for-byte. Treat math blocks as
supported opaque child blocks inside lists, blockquotes, divs, and footnotes.
Keep blank lines around display math according to normal block formatting.

Implementation checklist:

- [x] Parse `$$` display math blocks.
- [x] Parse `\[...\]` display math blocks.
- [x] Parse `\begin{...}` to matching `\end{...}` for common math
      environments.
- [x] Preserve body bytes and delimiters.

Example 1:

Before:

~~~~~md
$$
\begin{aligned}
a^2 + b^2 &= c^2
\end{aligned}
$$
~~~~~

After:

~~~~~md
$$
\begin{aligned}
a^2 + b^2 &= c^2
\end{aligned}
$$
~~~~~

Example 2:

Before:

~~~~~md
\[
E = mc^2
\]
~~~~~

After:

~~~~~md
\[
E = mc^2
\]
~~~~~

Example 3:

Before:

~~~~~md
- Use the identity:

  $$
  x^2 + 2x + 1 = (x + 1)^2
  $$
~~~~~

After:

~~~~~md
- Use the identity:

  $$
  x^2 + 2x + 1 = (x + 1)^2
  $$
~~~~~

## Suggested Implementation Order

1. Inline token support: link-label markup, inline HTML, strikethrough, and
   multiline links/images. These unlock paragraphs, footnotes, table cells, and
   many list items.
2. Code fence info parsing: Quarto executable fences, Pandoc attributes,
   MyST code-cell fences, and raw-format fences. These reduce skipped code
   fences and make rich child blocks easier.
3. Structural child block support: lists and blockquotes with nested children.
   This should reuse the code fence and inline work above.
4. Quarto/Pandoc blocks: fenced divs, shortcodes, includes, and display math.
   These should be represented as explicit block nodes rather than raw-block
   copies.
5. Tables and footnotes: update them after inline protected tokens are stable.

## Verification Plan

- Run focused CLI case tests after each feature.
- Run `cargo test` before merging a feature group.
- Run the fixture diagnostics command with `--features format-trace`.
- Compare:
  - total `markdown trace: skipped` count;
  - skipped counts by kind;
  - embedded formatter failure count;
  - any new hard formatter errors.
- Inspect representative diffs for changed fixture files. Do not accept broad
  reflow until the changed syntax class has direct tests.

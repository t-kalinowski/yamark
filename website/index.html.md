---
title: Yamark
description: An extremely fast formatter for YAML and Markdown, written in Rust.
toc: false
---

::: {.hero-shell}
::: {.hero-copy}
::: {.hero-kicker}
![](assets/favicon.svg){.hero-mark}
[Beta]{.status-chip}
:::

Yamark formats every layer of a Markdown file in one pass: prose rewraps, YAML
frontmatter tidies, and fenced code goes to its language's formatter -
recursively, all the way down. It formats Markdown wherever it lives: `.md` and
`.qmd` files, YAML scalars, and marked strings or comments in Python and R
source. When Yamark cannot prove a rewrite is safe, it leaves the file
unchanged.

::: {.hero-actions}
[Install](usage.qmd#install){.hero-button .primary}
[See examples](examples.qmd){.hero-button}
[Benchmarks](benchmarks.qmd){.hero-button}
:::
:::

::: {.terminal-window}
::: {.terminal-chrome}
<span>yamark format</span>
:::

```sh
$ yamark format config.yaml docs/
format config.yaml
format docs/index.md
format docs/reference.qmd
```
:::
:::

::: {.workflow-strip}
::: {.workflow-item}
**Format**

Rewrite supported files in place.
:::

::: {.workflow-item}
**Check**

Use non-mutating CI and diff modes.
:::

::: {.workflow-item}
**Embed**

Format marked Markdown in Python or R.
:::

::: {.workflow-item}
**Recurse**

Format YAML and Markdown inside fences.
:::
:::



## A quick example

A short Markdown file with YAML frontmatter - the shape of most skills, prompts,
and LLM-readable docs. The first pane is the file as typed; the second is what
`yamark format` writes back.



:::: {.before-after}
::: {.before-after-pane #demo-before}
**Before** <label class="softwrap-toggle"><input type="checkbox" id="demo-softwrap-toggle"> soft wrap</label>

```markdown
---
title: Why YAML + Markdown?
description: Token-efficient and human-readable, structured enough for programs and free-form enough for prose - the lingua franca between people and language models.
tags: [llm, authoring, formats]
---

# Why YAML + Markdown?

YAML and Markdown are the closest thing the web has to a native interface between humans and language models: cheap to tokenize, easy to read, and trivial to render. The frontmatter holds the metadata your code wants to query; the body holds the prose people and models want to write.

Where the combination shines:

- Skills and prompt files where typed frontmatter sits next to free-form instructions in one diffable file.
  - Agent skills, OpenAI custom GPT instructions, and Cursor rules all follow this shape.
  - The body can carry code samples and nested Markdown structures with their own escapes and indentation.
- Retrieval and RAG corpora that humans author and models consume without a build step.
- Tool inputs and outputs that round-trip through chat UIs and stay recognizable after the model rewrites them.
- Authoring pipelines that render the same source to HTML, PDF, and agent context, with no template engine in the middle.
```

:::

::: {.before-after-pane}
**After**

```markdown
---
title: Why YAML + Markdown?
description: >-
  Token-efficient and human-readable, structured enough for programs and
  free-form enough for prose - the lingua franca between people and
  language models.
tags: [llm, authoring, formats]
---

# Why YAML + Markdown?

YAML and Markdown are the closest thing the web has to a native
interface between humans and language models: cheap to tokenize, easy to
read, and trivial to render. The frontmatter holds the metadata your
code wants to query; the body holds the prose people and models want to
write.

Where the combination shines:

- Skills and prompt files where typed frontmatter sits next to free-form
  instructions in one diffable file.
  - Agent skills, OpenAI custom GPT instructions, and Cursor rules all
    follow this shape.
  - The body can carry code samples and nested Markdown structures with
    their own escapes and indentation.
- Retrieval and RAG corpora that humans author and models consume
  without a build step.
- Tool inputs and outputs that round-trip through chat UIs and stay
  recognizable after the model rewrites them.
- Authoring pipelines that render the same source to HTML, PDF, and
  agent context, with no template engine in the middle.
```
:::
::::

Toggle soft wrap on the Before pane to see what the raw file actually looks
like. Yamark wraps each region the way its grammar expects: folded scalars at
their column, prose at a comfortable width, and list continuations indented
under their bullets.

```{=html}
<script>
(function () {
  var toggle = document.getElementById('demo-softwrap-toggle');
  var pane = document.getElementById('demo-before');
  if (!toggle || !pane) return;
  var apply = function () { pane.classList.toggle('soft-wrap', toggle.checked); };
  toggle.addEventListener('change', apply);
  apply();
})();
</script>
```

::: {.feature-grid}
::: {.feature}
### One command

Run `yamark format` on a file, a folder, or the whole repository. The same
command also reads from stdin and runs in CI or editor integrations.
:::

::: {.feature}
### Rewrap after writing

Edit Markdown prose, YAML descriptions, prompt bodies, and front matter without
hand-maintaining line breaks.
:::

::: {.feature}
### Conservative writes

Reparse changed YAML regions, check targeted Markdown replacements, and avoid
rewriting unchanged files.
:::
:::

## Performance



**Yamark formats a 4 MB Markdown document in 115 ms and a 4 MB YAML file in 83 ms.** The next-fastest tool on each is `dprint-markdown` (338 ms) and `yamlfmt` (177 ms). On a directory of 500 YAML files (50 MB), yamark finishes in 123 ms; the next-fastest formatter, `deno-fmt`, takes 2.1 s.

Same harness, same input, every tool's own CLI used simply, and
the median of 3 measured runs after 1 warmup run. Full tables per input kind - Markdown, YAML,
Markdown with front matter, and a directory tree - plus methodology and
reproduce commands are on the [Benchmarks](benchmarks.qmd) page.

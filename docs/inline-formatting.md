# Inline formatting

`InlineContent` in `src/core/wrap/inline.rs` recognizes paragraph content with the existing inline scanners. It distinguishes editable gaps, protected source slices, real links, and supported markup. The block parser still owns block recognition and container prefixes.

## Planning and emission

1. Containers pass prefix-stripped content with authored line endings into `prepare_prefixed_markdown_lines`. Structural separators receive their own whitespace cleanup.
2. Inline recognition precedes whitespace normalization, link normalization, and canonicalization. Code, math, reference links, autolinks, brace spans, and LaTeX commands retain their source slices. Editable gaps supply word and hard-break boundaries.
3. Main's retained `Draft` stores normalized alternatives, measured token ranges, authored breaks, and link identity. Resolving effective options chooses an alternative and records layout in `Plan`. Emission executes the plan without reinterpreting Markdown.
4. Wrapping splits only tokens classified as real links and restores container prefixes inside multiline tokens. Finalized paragraph, container, and child Markdown output bypasses general line trimming, so recursive emission cannot remove literal spaces.

List support checks collect multiline ranges recursively through recognized markup. Continuation eligibility follows the existing grammar independently of emitted prefix width; prefix removal consumes only indentation present in the source. Heading canonicalization shares inline recognition, while heading and table spacing retain their existing rules.

## Compatibility and boundaries

Main's template policy remains in place, including eligible code templates and simple bare expressions in ordinary paragraphs. Its configured-delimiter, container, and standalone-placement restrictions still apply. Raw angles outside consumed literals or links continue to disable link normalization for that input. This change adds no HTML or template-language grammar.

The existing scanners still determine recognized boundaries, including their backtick closing-run limitations. Once they recognize a protected slice, normalization and canonicalization cannot change its contents. Multiline tokens remain atomic; layout does not optimize packing around their individual physical lines. Unsupported blockquote indentation and multiline brace spans retain their existing preservation behavior.

A follow-up such as #4 can reuse protected slices, gaps, parsed container content lines, and retained layout. It can remove independent literal-preservation guards and raw-prefix guesses for template placement. Broader template policy still needs the relevant configuration and must make placement decisions on original content lines before layout consumes those boundaries.

Public CLI tests check exact literal bytes, real-link compatibility, and repeated formatting. Generate the matching transcripts with `YAMARK_UPDATE_CASES=markdown_inline_preservation cargo test --test cli_cases`, then rerun without the variable. `--verify` checks YAML equivalence; it does not prove Markdown literal preservation.

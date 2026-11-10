# Open work

Record reproduced unresolved bugs and agreed follow-up work here, with the owning boundary and the evidence needed to close each item. Keep historical proposals in `design-sketches/`; they are not pending release requirements.

## File-scoped YAML directives override narrower scopes

After parsing, `YamlParser::parse` in `src/core/yaml.rs` replays the accumulated `scope=file` delta over every AST node and nested Markdown document. The replay overwrites a `scope=next` or `scope=from-here` directive that sets the same option or corrects the same abbreviation, even when that directive comes later. For example, after `# fmt: wrap=sentence scope=file`, a later `# fmt: wrap=none scope=next` has no effect. The replay also overwrites a `scope=file` directive inside a nested Markdown scalar. The Markdown parser patches existing nodes when it reaches a file-scoped directive, so a later directive takes precedence there.

The YAML parser's file-scope replay owns this. To close the item, add YAML transcripts in which a later `scope=next` and a later `scope=from-here` directive override a file-scoped one for `wrap`, `compact`, and `abbreviations`, as they do in Markdown, and update the "Scopes" section of the directive reference.

# Inline literal preservation

Paragraph preparation recognizes inline boundaries before changing whitespace.
Code and math recognized by the existing scanners keep their source bytes,
including spaces, tabs, escapes, and LF, CRLF, or CR line endings. Supported
surrounding prose, links, and emphasis still receive the requested formatting.
A multiline literal is one wrapping token and can exceed the requested width.

Container preparation removes structural prefixes and retains the remaining
content and authored line endings. In a definition list, four authored spaces
can precede content even when the formatted continuation prefix has five.
Whitespace after the structural indentation belongs to the content. The same
inline pipeline handles supported paragraphs, lists, quotes, definitions,
footnotes, and Markdown reached through existing nested document paths.

## Preparation and emission

The retained representation is a flat set of source ranges. Ordinary text and
gaps are implicit between ranges. It reuses the existing literal, link, and
emphasis scanners, including their caches and precedence rules.

Resolution normalizes only for the effective options. It builds no canonical
alternative when canonicalization is disabled, and no measured token array for
`--wrap none` or paragraph wrapping. Later file-scoped directives derive the
requested form from the retained source and recognition facts. Hard breaks
outside literals retain their existing normalization scopes; nested quotes with
`--wrap none` also retain authored line scopes.

Emission copies retained decisions and maps literal ranges into output
coordinates as it restores prefixes. Nested output carries those ranges through
the same verbatim-range mechanism used for explicitly preserved Markdown.
Editable gaps and structural blank lines still receive ordinary output cleanup.

## Boundaries

This does not expand the block grammar, template eligibility, HTML matching, or
malformed-delimiter rules. Recognition stops at existing opaque inline tokens;
it does not recursively interpret code or math inside HTML or strikethrough.
Block boundaries still limit which source belongs to one inline paragraph.

Automatic fallback retains its existing cleanup policy. For example, a list
starting with `100.` and a four-space continuation remains unsupported by the
existing list formatter. Multiline templates and opaque HTML can also fall back.
Unlike recognized literals and explicit preservation directives, automatic
fallback does not promise byte-for-byte preservation or idempotence after its
trailing-space cleanup. The CLI cases retain controls for these boundaries.

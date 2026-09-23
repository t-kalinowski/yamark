//! Source-backed inline boundaries. Ordinary text and gaps are implicit between
//! atoms; recognition does not allocate words, render alternatives, or measure.
use super::*;

#[derive(Debug, Clone, Copy)]
enum Kind {
    Literal,
    Protected,
    Link,
    Markup,
}

#[derive(Debug, Clone, Copy)]
struct Atom {
    span: SourceSpan,
    kind: Kind,
}

#[derive(Debug, Clone)]
pub(super) struct InlineSource {
    atoms: Box<[Atom]>,
    normalize_links: bool,
    simple: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Gaps {
    Preserve,
    Lines,
    Reflow,
}

pub(super) struct NormalizedInline {
    pub text: String,
    pub literals: Vec<Span>,
    pub tokens: Vec<(SourceSpan, usize)>,
    pub breaks: Vec<(usize, Option<MarkdownHardBreakMarker>)>,
}

impl InlineSource {
    pub(super) fn recognize(source: &str) -> Option<Self> {
        let mut atoms = Vec::new();
        let mut normalize_links = true;
        if simple_inline_tokens_supported(source) {
            return Some(Self {
                atoms: atoms.into_boxed_slice(),
                normalize_links,
                simple: true,
            });
        }
        let mut scan = InlineScan::new(source);
        let mut index = 0;
        while index < source.len() {
            let byte = source.as_bytes()[index];
            // Bytewise prose traversal keeps the common path inexpensive.
            if !matches!(
                byte,
                b'`' | b'$' | b'[' | b'!' | b'{' | b'~' | b'<' | b'\\' | b'*' | b'_'
            ) {
                index += 1;
                continue;
            }
            let rest = &source[index..];
            if escaped_at(source, index) && byte != b'`' {
                index += 1;
                continue;
            }
            let atom = if let Some(end) = scan.literal_span_end(index) {
                Some((end, Kind::Literal))
            } else if byte == b'`' {
                Some((
                    index + delimiter_run_len_at(source, index, b'`'),
                    Kind::Protected,
                ))
            } else if let Some(end) = reference_style_link_span_end(&mut scan, index)
                .or_else(|| balanced_brace_span_end(source, index))
                .or_else(|| latex_command_token_end(source, index))
            {
                if multiline_brace_span_at(source, index) {
                    return None;
                }
                Some((end, Kind::Protected))
            } else if let Some(end) = strikethrough_span_end(source, index)
                .or_else(|| paired_inline_html_span_end(source, index))
                .or_else(|| inline_html_tag_span_end(source, index))
                .or_else(|| emphasis_span_end(&mut scan, index))
            {
                // HTML and emphasis keep their existing opaque spacing policy.
                // This is not a recursive HTML formatter.
                if source[index..end].contains('<') {
                    normalize_links &= !unconsumed_angle(&source[index..end]);
                }
                Some((end, Kind::Markup))
            } else if let Some(end) = commonmark_autolink_span_end(source, index) {
                normalize_links = false;
                Some((end, Kind::Protected))
            } else if matches!(byte, b'[' | b'!') && (byte == b'[' || rest.starts_with("![")) {
                // Retain label boundaries before editable soft breaks join.
                // The existing tokenizer validates the normalized label below.
                Some((
                    link_or_bracket_span_end(&mut scan, index, false)?,
                    Kind::Link,
                ))
            } else {
                if byte == b'<' && !escaped_at(source, index) {
                    normalize_links = false;
                }
                if rest.starts_with("~~") && !escaped_at(source, index) {
                    return None;
                }
                None
            };
            if let Some((end, kind)) = atom {
                atoms.push(Atom {
                    span: SourceSpan::new(Span::new(index, end)),
                    kind,
                });
                index = end;
                if matches!(kind, Kind::Literal) && byte == b'`' {
                    // Match the existing link normalizer: leftover closing ticks
                    // cannot open another literal partway through that run.
                    index += source[index..]
                        .bytes()
                        .take_while(|byte| *byte == b'`')
                        .count();
                }
            } else {
                index += 1;
            }
        }
        Some(Self {
            atoms: atoms.into_boxed_slice(),
            normalize_links,
            simple: false,
        })
    }

    pub(super) fn normalize(
        &self,
        source: &str,
        gaps: Gaps,
        canonical: bool,
        measure: bool,
    ) -> Option<NormalizedInline> {
        // Hard breaks retain their existing normalization/layout scope. Only
        // recognized literals can contain a physical hard-break spelling
        // without ending that scope; opaque HTML does not gain new semantics.
        let mut segments = Vec::new();
        if gaps != Gaps::Preserve && (gaps == Gaps::Lines || has_hard_break(source)) {
            let mut start = 0;
            for line in markdown_lines(source) {
                let end = line.body_start + line.body.len();
                let atom = self.atoms.partition_point(|atom| atom.span.end() < end);
                if self.atoms.get(atom).is_some_and(|atom| {
                    matches!(atom.kind, Kind::Literal)
                        && atom.span.start() < end
                        && end < atom.span.end()
                }) {
                    continue;
                }
                let (body, marker) = markdown_hard_break_line_content(line.body);
                if marker.is_some() || gaps == Gaps::Lines && !line.newline.is_empty() {
                    let body = if marker.is_some() {
                        body
                    } else {
                        line.body.trim_end_matches([' ', '\t'])
                    };
                    segments.push((
                        Span::new(start, line.body_start + body.len()),
                        marker,
                        line.newline,
                    ));
                    start = line.body_start + line.full.len();
                }
            }
            if !segments.is_empty() && start < source.len() {
                segments.push((Span::new(start, source.len()), None, ""));
            }
        }
        if segments.is_empty() {
            return self.normalize_whole(source, gaps, canonical, measure, false);
        }
        let mut out = NormalizedInline {
            text: String::with_capacity(source.len()),
            literals: Vec::new(),
            tokens: Vec::new(),
            breaks: Vec::new(),
        };
        for (span, marker, newline) in segments {
            let facts = self.segment(source, span)?;
            let part = facts.normalize_whole(
                span.slice(source),
                gaps,
                canonical,
                measure,
                gaps == Gaps::Reflow,
            )?;
            let offset = out.text.len();
            out.text.push_str(&part.text);
            out.literals.extend(
                part.literals
                    .into_iter()
                    .map(|span| Span::new(offset + span.start, offset + span.end)),
            );
            out.tokens
                .extend(part.tokens.into_iter().map(|(span, width)| {
                    (
                        SourceSpan::new(Span::new(offset + span.start(), offset + span.end())),
                        width,
                    )
                }));
            if measure {
                out.breaks.push((out.tokens.len(), marker));
            }
            if let Some(marker) = marker {
                out.text.push_str(marker.suffix());
            }
            out.text.push_str(newline);
        }
        Some(out)
    }

    fn segment(&self, source: &str, span: Span) -> Option<Self> {
        let first = self
            .atoms
            .partition_point(|atom| atom.span.end() <= span.start);
        let mut atoms = Vec::new();
        for atom in &self.atoms[first..] {
            if atom.span.start() >= span.end {
                break;
            }
            let start = atom.span.start().max(span.start);
            let end = atom.span.end().min(span.end);
            if start == atom.span.start() && end == atom.span.end() {
                atoms.push(Atom {
                    span: SourceSpan::new(Span::new(start - span.start, end - span.start)),
                    kind: atom.kind,
                });
            } else {
                // Only a nonliteral atom can cross a hard break. Reuse the
                // existing recognizers on its cut source edges, before edits.
                let partial = Self::recognize(&source[start..end])?;
                atoms.extend(partial.atoms.into_iter().map(|atom| Atom {
                    span: SourceSpan::new(Span::new(
                        start - span.start + atom.span.start(),
                        start - span.start + atom.span.end(),
                    )),
                    kind: atom.kind,
                }));
            }
        }
        let simple =
            self.simple || atoms.is_empty() && simple_inline_tokens_supported(span.slice(source));
        Some(Self {
            atoms: atoms.into_boxed_slice(),
            normalize_links: self.normalize_links || !unconsumed_angle(span.slice(source)),
            simple,
        })
    }

    fn normalize_whole(
        &self,
        source: &str,
        gaps: Gaps,
        canonical: bool,
        measure: bool,
        linewise_markup: bool,
    ) -> Option<NormalizedInline> {
        let mut out = NormalizedInline {
            text: String::with_capacity(source.len()),
            literals: Vec::new(),
            tokens: Vec::new(),
            breaks: Vec::new(),
        };
        let mut scan = InlineScan::new(source);
        let mut atoms = self.atoms.iter().peekable();
        let mut index = 0;
        while index < source.len() {
            if let Some(atom) = atoms.peek().filter(|atom| atom.span.start() == index) {
                let atom = **atom;
                atoms.next();
                let text = atom.span.span().slice(source);
                let mut consumed = atom.span.end();
                match atom.kind {
                    Kind::Literal => {
                        let start = out.text.len();
                        out.text.push_str(text);
                        out.literals.push(Span::new(start, out.text.len()));
                    }
                    Kind::Link if self.normalize_links => {
                        if gaps != Gaps::Preserve && text.contains(['\n', '\r']) {
                            let spaces = normalize_spaces_preserving_protected_spans(text);
                            out.text
                                .push_str(&normalize_supported_links_and_images(&spaces));
                        } else if let Some((end, normalized)) =
                            normalize_link_or_image_at(&mut scan, index)
                        {
                            out.text.push_str(&normalized);
                            consumed = consumed.max(end);
                            out.text.push_str(&source[end..consumed]);
                        } else {
                            out.text.push_str(text);
                        }
                    }
                    Kind::Markup => {
                        let text = markup_line_gaps(text, linewise_markup);
                        if self.normalize_links {
                            out.text
                                .push_str(&normalize_supported_links_and_images(&text));
                        } else {
                            out.text.push_str(&text);
                        }
                    }
                    Kind::Link if gaps != Gaps::Preserve => {
                        out.text
                            .push_str(&normalize_spaces_preserving_protected_spans(text));
                    }
                    _ => out.text.push_str(text),
                }
                index = consumed;
                while atoms.peek().is_some_and(|atom| atom.span.start() < index) {
                    atoms.next();
                }
                continue;
            }
            if let Some((end, marker)) = gap_end(source, index) {
                let gap = &source[index..end];
                let (body, newline) = strip_final_newline(gap);
                if gaps == Gaps::Preserve {
                    if !newline.is_empty() || end == source.len() {
                        if let Some(marker) = marker {
                            out.text.push_str(
                                body.strip_suffix('\\')
                                    .unwrap_or(body)
                                    .trim_end_matches([' ', '\t']),
                            );
                            out.text.push_str(marker.suffix());
                        } else {
                            out.text.push_str(body.trim_end_matches([' ', '\t']));
                        }
                        out.text.push_str(newline);
                    } else {
                        out.text.push_str(gap);
                    }
                } else if marker.is_some() || gaps == Gaps::Lines && !newline.is_empty() {
                    if let Some(marker) = marker {
                        out.text.push_str(marker.suffix());
                    }
                    out.text.push_str(newline);
                } else if end < source.len()
                    && !out.text.is_empty()
                    && !out.text.ends_with([' ', '\r', '\n'])
                {
                    out.text.push(' ');
                }
                index = end;
                continue;
            }
            let start = index;
            let next_atom = atoms.peek().map_or(source.len(), |atom| atom.span.start());
            while index < next_atom {
                let byte = source.as_bytes()[index];
                if byte.is_ascii_whitespace() || byte == b'\\' && gap_end(source, index).is_some() {
                    break;
                }
                index += 1;
            }
            out.text.push_str(&source[start..index]);
        }
        if canonical && out.text.contains('_') {
            out.text = canonicalize_recognized_inline(&out.text, &out.literals, 0);
        }
        // Validate and tokenize with the existing scanner, using the retained
        // literal boundaries in the normalized buffer. No widths or token array
        // are needed for unwrapped output. This also retains main's malformed
        // delimiter eligibility and emphasis formed across editable soft breaks.
        if self.simple {
            if measure {
                out.tokens = simple_inline_tokens(&out.text);
            }
        } else {
            let mut scan = InlineScan::new(&out.text);
            scan.literals = &out.literals;
            let mut index = 0;
            while index < out.text.len() {
                if out.text.as_bytes()[index].is_ascii_whitespace() {
                    index += 1;
                    continue;
                }
                let end = inline_token_end(&mut scan, index)?;
                if measure {
                    out.tokens.push((
                        SourceSpan::new(Span::new(index, end)),
                        token_width(&out.text[index..end]),
                    ));
                }
                index = end;
            }
        }
        if measure {
            out.breaks.push((out.tokens.len(), None));
        }
        Some(out)
    }
}

// Preserve the existing physical-line cleanup within opaque markup. Its body
// is not recursively formatted or given new HTML/literal recognition rules.
fn markup_line_gaps(text: &str, join_lines: bool) -> Cow<'_, str> {
    if !text.contains(['\n', '\r']) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    for line in markdown_lines(text) {
        if join_lines {
            // Hard-break paragraph preparation has always normalized opaque
            // markup's physical lines before joining its soft breaks.
            let body = normalize_spaces_preserving_protected_spans(line.body.trim());
            if !body.is_empty() {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(&body);
            }
            continue;
        } else if line.newline.is_empty() {
            out.push_str(line.body);
        } else if let (body, Some(marker)) = markdown_hard_break_line_content(line.body) {
            out.push_str(body);
            out.push_str(marker.suffix());
        } else {
            out.push_str(line.body.trim_end_matches([' ', '\t']));
        }
        out.push_str(line.newline);
    }
    Cow::Owned(out)
}

// The normalizer's angle policy applies to the whole input. Inspect only the
// rare enclosing markup containing angles; consume literals and links in the
// same order as normalization, without introducing HTML matching.
fn unconsumed_angle(source: &str) -> bool {
    let mut scan = InlineScan::new(source);
    let mut index = 0;
    while index < source.len() {
        if source.as_bytes()[index] == b'<' && !escaped_at(source, index) {
            return true;
        }
        if let Some(end) = scan
            .literal_span_end(index)
            .or_else(|| balanced_brace_span_end(source, index))
            .or_else(|| reference_style_link_span_end(&mut scan, index))
            .or_else(|| link_or_bracket_token_end(&mut scan, index))
        {
            index = end;
        } else {
            index += source[index..]
                .chars()
                .next()
                .expect("source character")
                .len_utf8();
        }
    }
    false
}

fn gap_end(source: &str, start: usize) -> Option<(usize, Option<MarkdownHardBreakMarker>)> {
    let mut end = start;
    while source
        .as_bytes()
        .get(end)
        .is_some_and(|byte| byte.is_ascii_whitespace() && !matches!(byte, b'\r' | b'\n'))
    {
        end += 1;
    }
    let whitespace_end = end;
    if source.as_bytes().get(end) == Some(&b'\\')
        && (end + 1 == source.len()
            || matches!(source.as_bytes().get(end + 1), Some(b'\r' | b'\n')))
    {
        end += 1;
    }
    if end == source.len() || matches!(source.as_bytes().get(end), Some(b'\r' | b'\n')) {
        let line_start = source[..start]
            .rfind(['\n', '\r'])
            .map_or(0, |index| index + 1);
        let (_, marker) = markdown_hard_break_line_content(&source[line_start..end]);
        if source.as_bytes().get(end) == Some(&b'\r') {
            end += 1;
        }
        if source.as_bytes().get(end) == Some(&b'\n') {
            end += 1;
        }
        return (end > start).then_some((end, marker));
    }
    (whitespace_end > start).then_some((whitespace_end, None))
}

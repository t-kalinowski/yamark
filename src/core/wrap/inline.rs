use super::*;

/// Inline structure in paragraph content, after container prefixes are removed.
/// Gaps retain authored line endings until the wrapping policy consumes them.
/// Protected parts always borrow their complete, unmodified source slice.
pub(super) struct InlineContent<'a> {
    parts: Vec<InlinePart<'a>>,
    supported: bool,
    normalize_links: bool,
}

enum InlinePart<'a> {
    Text(&'a str),
    Gap(&'a str, Option<MarkdownHardBreakMarker>),
    Protected(&'a str),
    Link(&'a str, InlineLink<'a>),
    Emphasis {
        source: &'a str,
        span: EmphasisSpan,
        canonicalize: bool,
        content: InlineContent<'a>,
    },
    Markup {
        opening: &'a str,
        content: InlineContent<'a>,
        closing: &'a str,
    },
}

pub(super) struct InlineLink<'a> {
    label: String,
    image: bool,
    raw_target: &'a str,
    target: LinkTarget<'a>,
    attribute: String,
}

impl<'a> InlineLink<'a> {
    pub(super) fn parse(scan: &mut InlineScan<'a>, start: usize) -> Option<(usize, Self)> {
        let text = scan.text;
        let image = text[start..].starts_with("![");
        let label_start = start + if image { 2 } else { 1 };
        let label_close = scan.square_close(label_start)?;
        let after_label = label_close + 1;
        if text.as_bytes().get(after_label) != Some(&b'(') {
            return None;
        }
        let destination_close = find_simple_destination_close(text, after_label + 1)?;
        let label = normalize_link_label(&text[label_start..label_close], !image)?;
        let raw_target = &text[after_label + 1..destination_close];
        let target = parse_simple_link_target(raw_target)?;
        let mut end = destination_close + 1;
        let attribute = if image {
            normalize_image_attribute_after(text, end)
        } else {
            normalize_attribute_after(text, end)
        };
        let attribute = if let Some((attribute_end, attribute)) = attribute {
            end = attribute_end;
            attribute
        } else {
            String::new()
        };
        Some((
            end,
            Self {
                label,
                image,
                raw_target,
                target,
                attribute,
            },
        ))
    }

    pub(super) fn render(&self, preserve_lines: bool) -> String {
        let target = if preserve_lines && self.raw_target.contains(['\n', '\r']) {
            render_existing_split_link_target(self.raw_target, self.target)
        } else {
            render_link_target(self.target)
        };
        format!(
            "{}[{}]({target}){}",
            if self.image { "!" } else { "" },
            self.label,
            self.attribute
        )
    }
}

impl<'a> InlineContent<'a> {
    pub(super) fn parse(source: &'a str) -> Self {
        let mut scan = InlineScan::new(source);
        let mut parts = Vec::new();
        let mut supported = true;
        let mut normalize_links = true;
        let mut canonical_angle_end = 0;
        let mut index = 0;
        while index < source.len() {
            let rest = &source[index..];
            if let Some((end, marker)) = gap_end(source, index) {
                parts.push(InlinePart::Gap(&source[index..end], marker));
                index = end;
                continue;
            }
            // Escapes belong to prose. Consume the escaped character together
            // with its backslash so it cannot open a protected fragment.
            if rest.starts_with('\\') && escaped_at(source, index + 1) {
                let mut end = index + 2;
                if source.as_bytes()[index + 1] == b'`' {
                    end += source[end..]
                        .bytes()
                        .take_while(|byte| *byte == b'`')
                        .count();
                }
                parts.push(InlinePart::Text(&source[index..end]));
                index = end;
                continue;
            }
            // Keep main's raw-angle policy: only angles consumed by a literal
            // or a real link may participate in link normalization.
            normalize_links &= !rest.starts_with('<');
            // Retain the canonicalizer's existing angle-span boundary without
            // changing whitespace, wrapping, or HTML recognition inside it.
            if rest.starts_with('<')
                && let Some(close) = rest.find('>')
            {
                canonical_angle_end = index + close + 1;
            }
            if let Some(end) = immutable_span_end(&mut scan, index) {
                let text = &source[index..end];
                supported &= !multiline_brace_span_at(source, index);
                supported &= !(text.starts_with('\\') && !text.contains('{'));
                parts.push(InlinePart::Protected(text));
                index = end;
                continue;
            }
            if let Some((open_end, close_start, end)) = markup_span(source, index) {
                let content = Self::parse(&source[open_end..close_start]);
                normalize_links &= content.normalize_links;
                parts.push(InlinePart::Markup {
                    opening: &source[index..open_end],
                    content,
                    closing: &source[close_start..end],
                });
                index = end;
                continue;
            }
            if (rest.starts_with('[') || rest.starts_with("!["))
                && let Some((end, link)) = InlineLink::parse(&mut scan, index)
            {
                parts.push(InlinePart::Link(&source[index..end], link));
                index = end;
                continue;
            }
            if let Some(span) = emphasis_span_at(source, index) {
                let content = Self::parse(&source[index + span.run..span.close]);
                normalize_links &= content.normalize_links;
                let relative = EmphasisSpan {
                    close: span.close - index,
                    end: span.end - index,
                    run: span.run,
                };
                parts.push(InlinePart::Emphasis {
                    source: &source[index..span.end],
                    span: relative,
                    canonicalize: index >= canonical_angle_end,
                    content,
                });
                index = span.end;
                continue;
            }
            if (rest.starts_with('[') || rest.starts_with("!["))
                && let Some(end) = link_or_bracket_token_end(&mut scan, index)
            {
                parts.push(InlinePart::Protected(&source[index..end]));
                index = end;
                continue;
            }
            // An unmatched run is one literal opener, not a sequence of
            // progressively shorter candidate code delimiters. As before, the
            // outer emphasis/markup scanner determines its own eligibility.
            if rest.starts_with('`') {
                let end = index + rest.bytes().take_while(|byte| *byte == b'`').count();
                parts.push(InlinePart::Text(&source[index..end]));
                supported = false;
                index = end;
                continue;
            }
            supported &=
                !rest.starts_with(['$', '[']) && !rest.starts_with("![") && !rest.starts_with("~~");
            let ch = rest.chars().next().expect("index is on a char boundary");
            supported &= !ch.is_whitespace() || ch.is_ascii_whitespace();
            let end = index + ch.len_utf8();
            if let Some(InlinePart::Text(text)) = parts.last_mut() {
                let start = index - text.len();
                *text = &source[start..end];
            } else {
                parts.push(InlinePart::Text(&source[index..end]));
            }
            index = end;
        }
        Self {
            parts,
            supported,
            normalize_links,
        }
    }

    pub(super) fn render(&self, links: bool, canonical: bool) -> String {
        self.parts
            .iter()
            .map(|part| part.render(links && self.normalize_links, canonical, true, None))
            .collect()
    }

    fn render_markup(&self, links: bool, canonical: bool, line_gaps: Option<bool>) -> String {
        match line_gaps {
            // A trailing gap before a closing delimiter is inside the same
            // physical line. Only actual line endings receive line cleanup.
            Some(hard_breaks) => self.render_lines(links, canonical, hard_breaks, false),
            None => self.render(links, canonical),
        }
    }

    pub(super) fn supported(&self) -> bool {
        self.supported
    }

    pub(super) fn multiline_spans(&self, source: &str) -> Vec<std::ops::Range<usize>> {
        let mut spans = Vec::new();
        for part in &self.parts {
            match part {
                InlinePart::Protected(text) | InlinePart::Link(text, _)
                    if text.contains(['\n', '\r']) =>
                {
                    let start = text.as_ptr() as usize - source.as_ptr() as usize;
                    spans.push(start..start + text.len());
                }
                InlinePart::Emphasis { content, .. } | InlinePart::Markup { content, .. } => {
                    spans.extend(content.multiline_spans(source));
                }
                _ => {}
            }
        }
        spans
    }

    pub(super) fn preserve_lines(&self, links: bool, canonical: bool, hard_breaks: bool) -> String {
        self.render_lines(links, canonical, hard_breaks, true)
    }

    fn render_lines(
        &self,
        links: bool,
        canonical: bool,
        hard_breaks: bool,
        trim_final_gap: bool,
    ) -> String {
        let mut out = String::new();
        for (index, part) in self.parts.iter().enumerate() {
            if let InlinePart::Gap(text, marker) = part {
                if text.contains(['\n', '\r']) || trim_final_gap && index + 1 == self.parts.len() {
                    let (body, newline) = strip_final_newline(text);
                    let body = if hard_breaks && marker.is_some() {
                        body.strip_suffix('\\').unwrap_or(body)
                    } else {
                        body
                    };
                    // Only editable gaps are trimmed. Spaces and newlines in
                    // a protected part never pass through line normalization.
                    out.push_str(body.trim_end_matches([' ', '\t']));
                    if hard_breaks && let Some(marker) = marker {
                        out.push_str(marker.suffix());
                    }
                    out.push_str(newline);
                } else {
                    out.push_str(text);
                }
            } else {
                out.push_str(&part.render(
                    links && self.normalize_links,
                    canonical,
                    true,
                    Some(hard_breaks),
                ));
            }
        }
        out
    }

    pub(super) fn segments(&self, canonical: bool, preserve_lines: bool) -> Vec<InlineSegment<'a>> {
        let mut segments = Vec::new();
        let mut tokens = Vec::new();
        let mut current: Option<InlineToken<'a>> = None;
        for part in &self.parts {
            if let InlinePart::Gap(text, marker) = part {
                if let Some(token) = current.take() {
                    tokens.push(token);
                }
                if (marker.is_some() || preserve_lines && text.contains(['\n', '\r']))
                    && !tokens.is_empty()
                {
                    segments.push(InlineSegment {
                        tokens: std::mem::take(&mut tokens),
                        hard_break: *marker,
                    });
                }
                continue;
            }
            let text = part.render(self.normalize_links, canonical, false, Some(true));
            if let Some(token) = &mut current {
                token.text.to_mut().push_str(&text);
                token.width = token_width(token.text());
                token.splittable_link &= matches!(part, InlinePart::Text(_));
            } else {
                let mut token = InlineToken::new(text);
                token.splittable_link =
                    self.normalize_links && matches!(part, InlinePart::Link(..));
                current = Some(token);
            }
        }
        if let Some(token) = current {
            tokens.push(token);
        }
        if !tokens.is_empty() {
            segments.push(InlineSegment {
                tokens,
                hard_break: None,
            });
        }
        segments
    }
}

pub(super) struct InlineSegment<'a> {
    pub(super) tokens: Vec<InlineToken<'a>>,
    pub(super) hard_break: Option<MarkdownHardBreakMarker>,
}

impl<'a> InlinePart<'a> {
    fn render(
        &self,
        links: bool,
        canonical: bool,
        preserve_lines: bool,
        line_gaps: Option<bool>,
    ) -> Cow<'a, str> {
        match self {
            Self::Text(text) | Self::Protected(text) | Self::Gap(text, _) => Cow::Borrowed(text),
            Self::Link(source, link) => {
                if links {
                    Cow::Owned(link.render(preserve_lines))
                } else if preserve_lines {
                    Cow::Borrowed(source)
                } else {
                    normalize_spaces_preserving_protected_spans(source)
                }
            }
            Self::Emphasis {
                source,
                span,
                canonicalize,
                content,
            } => {
                let canonical = canonical && *canonicalize;
                let marker = if canonical && source.starts_with('_') {
                    "*".repeat(span.run)
                } else {
                    source[..span.run].to_owned()
                };
                Cow::Owned(format!(
                    "{marker}{}{marker}{}",
                    content.render_markup(links, canonical, line_gaps),
                    &source[span.close + span.run..]
                ))
            }
            Self::Markup {
                opening,
                content,
                closing,
            } => Cow::Owned(format!(
                "{opening}{}{closing}",
                content.render_markup(links, false, line_gaps)
            )),
        }
    }
}

pub(super) fn immutable_span_end(scan: &mut InlineScan<'_>, index: usize) -> Option<usize> {
    let source = scan.text;
    inline_code_span_end(source, index)
        .or_else(|| inline_math_span_end(source, index))
        .or_else(|| reference_style_link_span_end(scan, index))
        .or_else(|| commonmark_autolink_span_end(source, index))
        .or_else(|| balanced_brace_span_end(source, index))
        .or_else(|| latex_command_token_end(source, index))
}

fn markup_span(source: &str, index: usize) -> Option<(usize, usize, usize)> {
    if let Some(end) = strikethrough_span_end(source, index) {
        let close = index + 2 + source[index + 2..end].find("~~")?;
        return Some((index + 2, close, end));
    }
    if let Some(end) = paired_inline_html_span_end(source, index) {
        let open_end = index + source[index..end].find('>')? + 1;
        let close = index + source[index..end].rfind("</")?;
        return Some((open_end, close, end));
    }
    inline_html_tag_span_end(source, index).map(|end| (index + 1, end - 1, end))
}

fn gap_end(source: &str, start: usize) -> Option<(usize, Option<MarkdownHardBreakMarker>)> {
    let mut end = start;
    // Use the same ASCII whitespace class as prose reflow. Line endings are
    // consumed separately below so their authored bytes stay attached to the gap.
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

//! Opaque template recognition shared by inline preparation and policy checks.

use super::{
    InlineScan, balanced_brace_span_end, emphasis_span_end, escaped_at, inline_math_span_end,
    latex_command_token_end, link_or_bracket_token_end, strikethrough_span_end,
};
use crate::core::directives::TemplateDelimiter;
use crate::core::source::{SourceSpan, Span};

/// Original-source ranges used to reject unsupported template contexts before
/// preparing inline drafts. Expressions and code contents are opaque to policy.
#[derive(Debug, Clone)]
pub(crate) struct ProseTemplateSpans(Option<Vec<TemplateSpan>>);

#[derive(Debug, Clone, Copy)]
struct TemplateSpan {
    span: SourceSpan,
    expression: bool,
}

impl ProseTemplateSpans {
    pub(crate) fn recognize(source: &str, delimiters: &[TemplateDelimiter]) -> Self {
        Self(Self::recognize_contents(source, delimiters))
    }

    fn recognize_contents(
        source: &str,
        delimiters: &[TemplateDelimiter],
    ) -> Option<Vec<TemplateSpan>> {
        let mut scan = InlineScan::new(source);
        let mut contents = Vec::new();
        let mut index = 0;
        while index < source.len() {
            let rest = &source[index..];
            match inline_template_at(&mut scan, index, delimiters) {
                TemplateBoundary::Complete(end) => {
                    contents.push(TemplateSpan {
                        span: SourceSpan::new(Span::new(index, end)),
                        expression: true,
                    });
                    index = end;
                    continue;
                }
                TemplateBoundary::Incomplete => return None,
                TemplateBoundary::Absent => {}
            }
            if rest.starts_with('`') {
                if escaped_at(source, index) {
                    return None;
                }
                let run = rest.bytes().take_while(|byte| *byte == b'`').count();
                let end = scan.code_span_end(index)?;
                // The shared matcher can consume part of a longer closing run.
                // Reject that ambiguity here without changing its grammar.
                if source[end..].starts_with('`') || source[index..end].contains(['\n', '\r']) {
                    return None;
                }
                contents.push(TemplateSpan {
                    span: SourceSpan::new(Span::new(index + run, end - run)),
                    expression: false,
                });
                index = end;
                continue;
            }
            if !escaped_at(source, index) {
                // Do not broaden HTML, nested markup, or multiline literals.
                if rest.starts_with('<') {
                    return None;
                }
                let end = if rest.starts_with('[') || rest.starts_with("![") {
                    Some(link_or_bracket_token_end(&mut scan, index)?)
                } else if rest.starts_with('$') {
                    let end = inline_math_span_end(source, index)?;
                    // Existing passes disagree about escaped dollar closers and
                    // double-dollar runs. Do not admit those math boundaries.
                    if rest.starts_with("$$") || source[index + 1..end - 1].contains('$') {
                        return None;
                    }
                    Some(end)
                } else {
                    // Stop at an unmatched construct rather than rescanning its
                    // suffix at each later candidate opener.
                    let end = if rest.starts_with('{') {
                        Some(balanced_brace_span_end(source, index)?)
                    } else if rest.starts_with("~~") {
                        Some(strikethrough_span_end(source, index)?)
                    } else if rest.starts_with(['*', '_']) && !list_bullet_at(source, index) {
                        Some(emphasis_span_end(&mut scan, index)?)
                    } else {
                        latex_command_token_end(source, index)
                    };
                    if end.is_some_and(|end| source[index..end].contains(['`', '<'])) {
                        return None;
                    }
                    end
                };
                if let Some(end) = end {
                    if source[index..end].contains(['\n', '\r']) {
                        return None;
                    }
                    index = end;
                    continue;
                }
            }
            index += rest.chars().next().expect("source character").len_utf8();
        }
        Some(contents)
    }

    pub(crate) fn contains_all_templates(
        &self,
        source: &str,
        delimiters: &[TemplateDelimiter],
    ) -> bool {
        let Some(contents) = &self.0 else {
            return false;
        };
        delimiters.iter().all(|delimiter| {
            // Template expressions may quote either marker any number of times.
            // Code keeps its existing requirement that both markers occur within
            // the same code span; fragments in separate literals cannot pair.
            let mut opens = markers_outside_expressions(source, &delimiter.open, contents);
            let mut closes = markers_outside_expressions(source, &delimiter.close, contents);
            let mut spans = contents.iter().filter(|span| !span.expression);
            let mut span = spans.next();
            loop {
                let open = opens.next();
                let close = if delimiter.open == delimiter.close {
                    opens.next()
                } else {
                    closes.next()
                };
                let (open, close) = match (open, close) {
                    (None, None) => return true,
                    (Some(open), Some(close)) if open + delimiter.open.len() <= close => {
                        (open, close)
                    }
                    _ => return false,
                };
                while span.is_some_and(|span| span.span.end() <= open) {
                    span = spans.next();
                }
                if !span.is_some_and(|span| {
                    span.span.start() <= open && close + delimiter.close.len() <= span.span.end()
                }) {
                    return false;
                }
            }
        })
    }
}

fn markers_outside_expressions<'a>(
    source: &'a str,
    marker: &'a str,
    contents: &'a [TemplateSpan],
) -> impl Iterator<Item = usize> + 'a {
    let mut expressions = contents.iter().filter(|span| span.expression);
    let mut expression = expressions.next();
    source
        .match_indices(marker)
        .filter(move |(start, _)| {
            while expression.is_some_and(|span| span.span.end() <= *start) {
                expression = expressions.next();
            }
            !expression.is_some_and(|span| {
                span.span.start() <= *start && *start + marker.len() <= span.span.end()
            })
        })
        .map(|(start, _)| start)
}

fn list_bullet_at(source: &str, index: usize) -> bool {
    source.as_bytes().get(index) == Some(&b'*')
        && matches!(source.as_bytes().get(index + 1), Some(b' ' | b'\t'))
        && source[..index]
            .rsplit(['\n', '\r'])
            .next()
            .unwrap()
            .trim_matches([' ', '\t', '>'])
            .is_empty()
}

pub(super) enum TemplateBoundary {
    Absent,
    Incomplete,
    Complete(usize),
}

// All inline passes share delimiter precedence and incomplete-token rejection.
// Competing pairs advance together so a missing closer cannot repeatedly scan
// the suffix before another pair completes at each subsequent opener.
pub(super) fn inline_template_at(
    scan: &mut InlineScan<'_>,
    index: usize,
    delimiters: &[TemplateDelimiter],
) -> TemplateBoundary {
    let source = scan.text;
    let mut matches = delimiters.iter().filter(|delimiter| {
        source[index..].starts_with(&delimiter.open)
            && !(delimiter.open == "{{"
                && delimiter.close == "}}"
                && (source[index..].starts_with("{{<") || source[index..].starts_with("{{%")))
    });
    let Some(first) = matches.next() else {
        return TemplateBoundary::Absent;
    };
    if escaped_at(source, index) {
        return TemplateBoundary::Absent;
    }
    let end = if let Some(second) = matches.next() {
        let mut candidates = std::iter::once(first)
            .chain(std::iter::once(second))
            .chain(matches)
            .filter_map(|delimiter| TemplateCandidate::new(scan, index, delimiter))
            .collect::<Vec<_>>();
        template_candidates_end(source, &mut candidates)
    } else if let Some(candidate) = TemplateCandidate::new(scan, index, first) {
        template_candidates_end(source, &mut [candidate])
    } else {
        None
    };
    if let Some(end) = end {
        TemplateBoundary::Complete(end)
    } else if source[index..].starts_with("{#")
        && delimiters.iter().any(literal_template_comment)
        && balanced_brace_span_end(source, index).is_some()
    {
        // Pandoc identifiers share the comment opener, but close with `}`.
        TemplateBoundary::Absent
    } else {
        TemplateBoundary::Incomplete
    }
}

fn literal_template_comment(delimiter: &TemplateDelimiter) -> bool {
    delimiter.open == "{#" && delimiter.close == "#}"
}

struct TemplateCandidate<'a> {
    delimiter: &'a TemplateDelimiter,
    start: usize,
    braces: usize,
    quote: Option<char>,
    escaped: bool,
    active: bool,
    literal: bool,
}

impl<'a> TemplateCandidate<'a> {
    fn new(
        scan: &mut InlineScan<'_>,
        start: usize,
        delimiter: &'a TemplateDelimiter,
    ) -> Option<Self> {
        if delimiter.open.contains(['\n', '\r']) || delimiter.close.contains(['\n', '\r']) {
            return None;
        }
        let start = start + delimiter.open.len();
        let literal = literal_template_comment(delimiter);
        if literal {
            // Failed comment searches can fall back to Pandoc attributes. Index
            // closers once so successive attributes do not rescan the suffix.
            let closes = scan.template_comment_closes.get_or_insert_with(|| {
                let mut previous = 0;
                let mut line_start = 0;
                scan.text
                    .match_indices("#}")
                    .map(|(index, _)| {
                        if let Some(newline) = scan.text[previous..index].rfind(['\n', '\r']) {
                            line_start = previous + newline + 1;
                        }
                        previous = index + 2;
                        (line_start, index)
                    })
                    .collect()
            });
            let (line_start, _) =
                closes.get(closes.partition_point(|(_, close)| *close < start))?;
            if *line_start > start {
                return None;
            }
        }
        Some(Self {
            delimiter,
            start,
            braces: 0,
            quote: None,
            escaped: false,
            active: true,
            literal,
        })
    }

    // This transition runs for every template character.
    #[inline]
    fn advance(&mut self, source: &str, index: usize, ch: char) -> Option<usize> {
        if !self.active || index < self.start {
            return None;
        }
        if self.literal || (self.quote.is_none() && self.braces == 0) {
            if source[index..].starts_with(&self.delimiter.close) {
                return Some(index + self.delimiter.close.len());
            }
            if self.literal {
                return None;
            }
        }
        if let Some(marker) = self.quote {
            if self.escaped {
                self.escaped = false;
            } else if ch == '\\' {
                self.escaped = true;
            } else if ch == marker {
                self.quote = None;
            }
        } else {
            match ch {
                '\'' | '"' => self.quote = Some(ch),
                '{' => self.braces += 1,
                '}' if self.braces == 0 => self.active = false,
                '}' => self.braces -= 1,
                _ => {}
            }
        }
        None
    }
}

fn template_candidates_end(
    source: &str,
    candidates: &mut [TemplateCandidate<'_>],
) -> Option<usize> {
    let start = candidates.iter().map(|candidate| candidate.start).min()?;
    for (offset, ch) in source[start..].char_indices() {
        if matches!(ch, '\n' | '\r') {
            return None;
        }
        let mut active = false;
        for candidate in &mut *candidates {
            if let Some(end) = candidate.advance(source, start + offset, ch) {
                return Some(end);
            }
            active |= candidate.active;
        }
        if !active {
            return None;
        }
    }
    None
}

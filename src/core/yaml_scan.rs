use crate::core::source::{LineEnding, SourceBuffer, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlLineScan {
    pub range: Span,
    pub start_line: usize,
    pub end_line: usize,
    pub source_scans: usize,
    pub scanned_lines: usize,
    pub tab_indentation: Option<YamlTabIndentation>,
    pub lines: Vec<YamlScannedLine>,
}

impl YamlLineScan {
    pub fn has_tab_indentation(&self, source: &SourceBuffer) -> Option<YamlTabIndentation> {
        if let Some(tab) = self.tab_indentation {
            return Some(tab);
        }
        self.lines.iter().find_map(|line| {
            let text = source.slice(line.content);
            tab_in_indentation(text).map(|column| YamlTabIndentation {
                line_index: line.index,
                column,
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlTabIndentation {
    pub line_index: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlScannedLine {
    pub index: usize,
    pub full: Span,
    pub content: Span,
    pub ending: LineEnding,
    pub indent: usize,
    pub kind: YamlLineKind,
    pub indicators: Vec<YamlIndicator>,
    pub properties: Vec<YamlPropertyToken>,
    pub scalar: Option<YamlScalarToken>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlLineKind {
    Blank,
    Comment,
    Directive,
    DocumentMarker,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlIndicator {
    pub kind: YamlIndicatorKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlIndicatorKind {
    SequenceEntry,
    MappingValue,
    FlowSequenceStart,
    FlowSequenceEnd,
    FlowMappingStart,
    FlowMappingEnd,
    FlowEntrySeparator,
    ExplicitKey,
    BlockScalarHeader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlPropertyToken {
    pub kind: YamlPropertyTokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlPropertyTokenKind {
    Tag,
    Anchor,
    Alias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlScalarToken {
    pub kind: YamlScalarTokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlScalarTokenKind {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    BlockScalarHeader,
}

pub fn scan_yaml_lines(source: &SourceBuffer, range: Span) -> YamlLineScan {
    let start_line = source
        .lines
        .partition_point(|line| line.full.end() <= range.start);
    let end_line = source
        .lines
        .partition_point(|line| line.full.start() < range.end);
    let mut lines = Vec::with_capacity(end_line.saturating_sub(start_line));
    let mut tab_indentation = None;

    for index in start_line..end_line {
        let line = source.lines[index];
        let text = source.line_text(index);
        if tab_indentation.is_none()
            && let Some(column) = tab_in_indentation(text)
        {
            tab_indentation = Some(YamlTabIndentation {
                line_index: index,
                column,
            });
        }
        let indent = indentation(text);
        let kind = classify_line(text);
        let indicators = scan_indicators(line.text.into(), text, indent);
        let properties = scan_property_tokens(line.text.into(), text, indent);
        let scalar = scan_scalar_token(
            source,
            line.text.into(),
            text,
            indent,
            &indicators,
            &properties,
        );
        lines.push(YamlScannedLine {
            index,
            full: line.full.into(),
            content: line.text.into(),
            ending: line.ending,
            indent,
            kind,
            indicators,
            properties,
            scalar,
        });
    }

    YamlLineScan {
        range,
        start_line,
        end_line,
        source_scans: 1,
        scanned_lines: lines.len(),
        tab_indentation,
        lines,
    }
}

pub(crate) fn scan_yaml_lines_basic(source: &SourceBuffer, range: Span) -> YamlLineScan {
    let start_line = source
        .lines
        .partition_point(|line| line.full.end() <= range.start);
    let end_line = source
        .lines
        .partition_point(|line| line.full.start() < range.end);
    let mut tab_indentation = None;

    for index in start_line..end_line {
        let text = source.line_text(index);
        if let Some(column) = tab_in_indentation(text) {
            tab_indentation = Some(YamlTabIndentation {
                line_index: index,
                column,
            });
            break;
        }
    }

    YamlLineScan {
        range,
        start_line,
        end_line,
        source_scans: 1,
        scanned_lines: end_line.saturating_sub(start_line),
        tab_indentation,
        lines: Vec::new(),
    }
}

fn classify_line(text: &str) -> YamlLineKind {
    let trimmed = text.trim_start_matches('\u{feff}').trim();
    if trimmed.is_empty() {
        YamlLineKind::Blank
    } else if document_marker_line(trimmed, "---") || document_marker_line(trimmed, "...") {
        YamlLineKind::DocumentMarker
    } else if trimmed
        .strip_prefix('#')
        .is_some_and(|rest| rest.trim_start().starts_with("fmt:"))
        || trimmed.starts_with("%YAML ")
        || trimmed.starts_with("%TAG ")
    {
        YamlLineKind::Directive
    } else if trimmed.starts_with('#') {
        YamlLineKind::Comment
    } else {
        YamlLineKind::Other
    }
}

fn document_marker_line(text: &str, marker: &str) -> bool {
    let Some(rest) = text.strip_prefix(marker) else {
        return false;
    };
    rest.is_empty()
        || rest
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_whitespace())
}

fn scan_indicators(line_span: Span, text: &str, indent: usize) -> Vec<YamlIndicator> {
    let bytes = text.as_bytes();
    let mut indicators = Vec::new();
    let mut quote = QuoteState::None;
    let mut escaped = false;
    let content_start = indent.min(bytes.len());

    let mut i = content_start;
    while i < bytes.len() {
        let byte = bytes[i];
        match quote {
            QuoteState::None => {
                let kind = match byte {
                    b'\'' => {
                        quote = QuoteState::Single;
                        None
                    }
                    b'"' => {
                        quote = QuoteState::Double;
                        escaped = false;
                        None
                    }
                    b'#' => break,
                    b'-' if i == content_start
                        && bytes
                            .get(i + 1)
                            .is_none_or(|next| next.is_ascii_whitespace()) =>
                    {
                        Some(YamlIndicatorKind::SequenceEntry)
                    }
                    b':' => Some(YamlIndicatorKind::MappingValue),
                    b'[' => Some(YamlIndicatorKind::FlowSequenceStart),
                    b']' => Some(YamlIndicatorKind::FlowSequenceEnd),
                    b'{' => Some(YamlIndicatorKind::FlowMappingStart),
                    b'}' => Some(YamlIndicatorKind::FlowMappingEnd),
                    b',' => Some(YamlIndicatorKind::FlowEntrySeparator),
                    b'?' if i == content_start
                        && bytes
                            .get(i + 1)
                            .is_none_or(|next| next.is_ascii_whitespace()) =>
                    {
                        Some(YamlIndicatorKind::ExplicitKey)
                    }
                    b'|' | b'>' if scalar_token_start(bytes, content_start, i) => {
                        Some(YamlIndicatorKind::BlockScalarHeader)
                    }
                    _ => None,
                };
                if let Some(kind) = kind {
                    indicators.push(YamlIndicator {
                        kind,
                        span: Span::new(line_span.start + i, line_span.start + i + 1),
                    });
                }
            }
            QuoteState::Single => {
                if byte == b'\'' {
                    quote = QuoteState::None;
                }
            }
            QuoteState::Double => {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    quote = QuoteState::None;
                }
            }
        }
        i += 1;
    }

    indicators
}

fn scan_property_tokens(line_span: Span, text: &str, indent: usize) -> Vec<YamlPropertyToken> {
    let bytes = text.as_bytes();
    let content_start = indent.min(bytes.len());
    let mut properties = Vec::new();
    let mut expect_node_start = true;
    let mut flow_depth = 0usize;
    let mut i = content_start;

    while i < bytes.len() {
        match bytes[i] {
            byte if byte.is_ascii_whitespace() => {
                i += 1;
            }
            b'#' => break,
            b'\'' => {
                i = quoted_end(bytes, i, b'\'');
                expect_node_start = false;
            }
            b'"' => {
                i = double_quoted_end(bytes, i);
                expect_node_start = false;
            }
            b'-' if i == content_start
                && bytes
                    .get(i + 1)
                    .is_none_or(|next| next.is_ascii_whitespace()) =>
            {
                i += 1;
                expect_node_start = true;
            }
            b'?' if i == content_start
                && bytes
                    .get(i + 1)
                    .is_none_or(|next| next.is_ascii_whitespace()) =>
            {
                i += 1;
                expect_node_start = true;
            }
            b'[' | b'{' => {
                flow_depth += 1;
                i += 1;
                expect_node_start = true;
            }
            b',' => {
                i += 1;
                expect_node_start = flow_depth > 0;
            }
            b':' if colon_starts_mapping_value(bytes, i) => {
                i += 1;
                expect_node_start = true;
            }
            b':' => {
                i += 1;
                expect_node_start = false;
            }
            b']' | b'}' => {
                flow_depth = flow_depth.saturating_sub(1);
                i += 1;
                expect_node_start = false;
            }
            b'!' if expect_node_start => {
                let end = tag_token_end(bytes, i);
                properties.push(YamlPropertyToken {
                    kind: YamlPropertyTokenKind::Tag,
                    span: Span::new(line_span.start + i, line_span.start + end),
                });
                i = end;
                expect_node_start = true;
            }
            b'&' if expect_node_start => {
                let end = anchor_or_alias_token_end(bytes, i);
                properties.push(YamlPropertyToken {
                    kind: YamlPropertyTokenKind::Anchor,
                    span: Span::new(line_span.start + i, line_span.start + end),
                });
                i = end;
                expect_node_start = true;
            }
            b'*' if expect_node_start => {
                let end = anchor_or_alias_token_end(bytes, i);
                properties.push(YamlPropertyToken {
                    kind: YamlPropertyTokenKind::Alias,
                    span: Span::new(line_span.start + i, line_span.start + end),
                });
                i = end;
                expect_node_start = false;
            }
            _ => {
                i += 1;
                expect_node_start = false;
            }
        }
    }

    properties
}

fn colon_starts_mapping_value(bytes: &[u8], index: usize) -> bool {
    bytes
        .get(index + 1)
        .is_none_or(|byte| byte.is_ascii_whitespace() || matches!(byte, b',' | b']' | b'}'))
}

fn tag_token_end(bytes: &[u8], start: usize) -> usize {
    if bytes.get(start + 1) == Some(&b'<') {
        let mut i = start + 2;
        while i < bytes.len() {
            if bytes[i] == b'>' {
                return i + 1;
            }
            i += 1;
        }
        return bytes.len();
    }
    let mut i = start + 1;
    while i < bytes.len() && !property_token_delimiter(bytes[i]) {
        i += 1;
    }
    i
}

fn anchor_or_alias_token_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() && !property_token_delimiter(bytes[i]) && bytes[i] != b':' {
        i += 1;
    }
    i
}

fn property_token_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'#' | b',' | b'[' | b']' | b'{' | b'}')
}

fn scan_scalar_token(
    source: &SourceBuffer,
    line_span: Span,
    text: &str,
    indent: usize,
    indicators: &[YamlIndicator],
    properties: &[YamlPropertyToken],
) -> Option<YamlScalarToken> {
    let bytes = text.as_bytes();
    let mut start = indent.min(bytes.len());
    if bytes.get(start) == Some(&b'-') {
        start += 1;
        while bytes
            .get(start)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            start += 1;
        }
    }
    if bytes.get(start) == Some(&b'?') {
        start += 1;
        while bytes
            .get(start)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            start += 1;
        }
    }
    if let Some(colon) = indicators
        .iter()
        .find(|indicator| indicator.kind == YamlIndicatorKind::MappingValue)
    {
        let relative_colon = colon.span.start.saturating_sub(line_span.start);
        if relative_colon >= start {
            start = relative_colon + 1;
            while bytes
                .get(start)
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                start += 1;
            }
        }
    }
    while bytes
        .get(start)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        start += 1;
    }
    start = skip_property_tokens_at_start(bytes, line_span, start, properties);
    if start >= bytes.len() || bytes[start] == b'#' {
        return None;
    }

    let (kind, end) = match bytes[start] {
        b'\'' => (
            YamlScalarTokenKind::SingleQuoted,
            quoted_end(bytes, start, b'\''),
        ),
        b'"' => (
            YamlScalarTokenKind::DoubleQuoted,
            double_quoted_end(bytes, start),
        ),
        b'|' | b'>' if scalar_token_start(bytes, indent.min(bytes.len()), start) => {
            let end = bytes[start..]
                .iter()
                .position(|byte| byte.is_ascii_whitespace() || *byte == b'#')
                .map(|offset| start + offset)
                .unwrap_or(bytes.len());
            (YamlScalarTokenKind::BlockScalarHeader, end)
        }
        _ => {
            let end = plain_scalar_end(bytes, start);
            (YamlScalarTokenKind::Plain, end)
        }
    };
    let span = Span::new(line_span.start + start, line_span.start + end);
    (!source.slice(span).is_empty()).then_some(YamlScalarToken { kind, span })
}

fn skip_property_tokens_at_start(
    bytes: &[u8],
    line_span: Span,
    mut start: usize,
    properties: &[YamlPropertyToken],
) -> usize {
    loop {
        while bytes
            .get(start)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            start += 1;
        }
        let absolute = line_span.start + start;
        let Some(property) = properties
            .iter()
            .find(|property| property.span.start == absolute)
        else {
            return start;
        };
        start = property.span.end.saturating_sub(line_span.start);
    }
}

fn scalar_token_start(bytes: &[u8], content_start: usize, index: usize) -> bool {
    if index == content_start {
        return true;
    }
    let mut i = content_start;
    if bytes.get(i) == Some(&b'-') || bytes.get(i) == Some(&b'?') {
        i += 1;
        while bytes.get(i).is_some_and(|byte| byte.is_ascii_whitespace()) {
            i += 1;
        }
    }
    if i == index {
        return true;
    }
    bytes[..index]
        .iter()
        .rposition(|byte| *byte == b':')
        .is_some_and(|colon| bytes[colon + 1..index].iter().all(u8::is_ascii_whitespace))
}

fn quoted_end(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == quote {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

fn double_quoted_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    let mut escaped = false;
    while i < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[i] == b'\\' {
            escaped = true;
        } else if bytes[i] == b'"' {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

fn plain_scalar_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    let mut end = bytes.len();
    while i < bytes.len() {
        if bytes[i] == b'#' && (i == start || bytes[i - 1].is_ascii_whitespace()) {
            end = i;
            break;
        }
        i += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    end
}

fn indentation(text: &str) -> usize {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    text.bytes().take_while(|byte| *byte == b' ').count()
}

fn tab_in_indentation(text: &str) -> Option<usize> {
    for (column, byte) in text.bytes().enumerate() {
        match byte {
            b' ' => {}
            b'\t' => return Some(column),
            _ => return None,
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteState {
    None,
    Single,
    Double,
}

use std::marker::PhantomData;
use std::num::NonZeroU32;

use memchr::memchr2;

pub const MAX_SOURCE_SPAN_OFFSET: usize = u32::MAX as usize - 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn empty(at: usize) -> Self {
        Self { start: at, end: at }
    }

    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub const fn start(self) -> usize {
        self.start
    }

    pub const fn end(self) -> usize {
        self.end
    }

    pub fn slice(self, source: &str) -> &str {
        &source[self.start..self.end]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceSpan<'src> {
    start_plus_one: NonZeroU32,
    end: u32,
    source: PhantomData<&'src str>,
}

impl<'src> SourceSpan<'src> {
    pub(crate) fn new(span: Span) -> Self {
        assert!(
            span.start <= span.end,
            "source span start must not exceed end"
        );
        assert!(
            span.end <= MAX_SOURCE_SPAN_OFFSET,
            "source span exceeds u32::MAX bytes"
        );
        Self {
            start_plus_one: NonZeroU32::new(span.start as u32 + 1)
                .expect("source span start offset is stored one-based"),
            end: span.end as u32,
            source: PhantomData,
        }
    }

    pub(crate) fn empty(at: usize) -> Self {
        Self::new(Span::empty(at))
    }

    pub const fn span(self) -> Span {
        Span {
            start: self.start(),
            end: self.end as usize,
        }
    }

    pub const fn start(self) -> usize {
        self.start_plus_one.get() as usize - 1
    }

    pub const fn end(self) -> usize {
        self.end as usize
    }

    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start() as u32) as usize
    }

    pub const fn is_empty(self) -> bool {
        self.start() == self.end as usize
    }

    pub(crate) fn set_start(&mut self, start: usize) {
        assert!(
            start <= self.end as usize,
            "source span start must not exceed end"
        );
        assert!(
            start <= MAX_SOURCE_SPAN_OFFSET,
            "source span start exceeds supported maximum"
        );
        self.start_plus_one = NonZeroU32::new(start as u32 + 1)
            .expect("source span start offset is stored one-based");
    }

    pub(crate) fn retag<'dst>(self) -> SourceSpan<'dst> {
        SourceSpan {
            start_plus_one: self.start_plus_one,
            end: self.end,
            source: PhantomData,
        }
    }

    pub fn as_str(self, source: &'src SourceBuffer) -> &'src str {
        source.slice(self)
    }
}

impl From<SourceSpan<'_>> for Span {
    fn from(span: SourceSpan<'_>) -> Self {
        span.span()
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self::new(value.start, value.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
    Cr,
    None,
}

impl LineEnding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
            Self::Cr => "\r",
            Self::None => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub full: SourceSpan<'static>,
    pub text: SourceSpan<'static>,
    pub ending: LineEnding,
}

#[derive(Debug, Clone)]
pub struct SourceBuffer {
    text: String,
    pub bom: Option<Span>,
    pub lines: Vec<Line>,
    pub dominant_line_ending: LineEnding,
}

impl SourceBuffer {
    pub fn new(text: String) -> Self {
        let bom = text
            .starts_with('\u{feff}')
            .then_some(Span::new(0, '\u{feff}'.len_utf8()));
        let mut lines = Vec::new();
        let mut cursor = 0usize;
        let bytes = text.as_bytes();
        let mut counts = [0usize; 3];

        while cursor < text.len() {
            let start = cursor;
            cursor = memchr2(b'\r', b'\n', &bytes[cursor..])
                .map(|offset| cursor + offset)
                .unwrap_or(text.len());
            let text_end = cursor;
            let ending = if cursor == text.len() {
                LineEnding::None
            } else if bytes[cursor] == b'\r'
                && cursor + 1 < text.len()
                && bytes[cursor + 1] == b'\n'
            {
                cursor += 2;
                counts[1] += 1;
                LineEnding::Crlf
            } else if bytes[cursor] == b'\r' {
                cursor += 1;
                counts[2] += 1;
                LineEnding::Cr
            } else {
                cursor += 1;
                counts[0] += 1;
                LineEnding::Lf
            };
            lines.push(Line {
                full: SourceSpan::new(Span::new(start, cursor)),
                text: SourceSpan::new(Span::new(start, text_end)),
                ending,
            });
        }

        let dominant_line_ending = if counts[1] > counts[0] && counts[1] >= counts[2] {
            LineEnding::Crlf
        } else if counts[2] > counts[0] && counts[2] > counts[1] {
            LineEnding::Cr
        } else if counts[0] > 0 {
            LineEnding::Lf
        } else {
            LineEnding::None
        };

        Self {
            text,
            bom,
            lines,
            dominant_line_ending,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn slice(&self, span: impl Into<Span>) -> &str {
        span.into().slice(&self.text)
    }

    pub fn line_text(&self, index: usize) -> &str {
        self.slice(self.lines[index].text)
    }

    pub fn line_full(&self, index: usize) -> &str {
        self.slice(self.lines[index].full)
    }

    pub fn line_at_byte(&self, byte: usize) -> usize {
        match self
            .lines
            .binary_search_by_key(&byte, |line| line.full.start())
        {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        }
    }

    pub fn line_column_at_byte(&self, byte: usize) -> (usize, usize) {
        let line_index = self.line_at_byte(byte);
        let line = self.lines.get(line_index).copied().unwrap_or(Line {
            full: SourceSpan::empty(0),
            text: SourceSpan::empty(0),
            ending: LineEnding::None,
        });
        let column = self.text[line.text.start().min(byte)..byte.min(line.text.end())]
            .chars()
            .count()
            + 1;
        (line_index + 1, column)
    }
}

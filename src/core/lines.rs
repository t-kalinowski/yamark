#[derive(Debug, Clone, Copy)]
pub(crate) struct TextLine<'a> {
    pub full: &'a str,
    pub body: &'a str,
    pub newline: &'a str,
    pub body_start: usize,
}

pub(crate) fn text_lines(source: &str) -> impl Iterator<Item = TextLine<'_>> {
    let mut start = 0;
    std::iter::from_fn(move || {
        if start == source.len() {
            return None;
        }
        let end = memchr::memchr2(b'\r', b'\n', &source.as_bytes()[start..])
            .map_or(source.len(), |offset| start + offset);
        let newline = if source[end..].starts_with("\r\n") {
            "\r\n"
        } else if end < source.len() {
            &source[end..end + 1]
        } else {
            ""
        };
        let full_end = end + newline.len();
        let line = TextLine {
            full: &source[start..full_end],
            body: &source[start..end],
            newline,
            body_start: start,
        };
        start = full_end;
        Some(line)
    })
}

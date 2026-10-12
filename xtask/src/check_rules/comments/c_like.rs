use crate::text::line_at;

#[derive(Clone, Copy)]
pub enum Markers {
    LineAndBlock,
    BlockOnly,
}

pub fn first_comment_line(text: &str, markers: Markers) -> Option<usize> {
    comment_start(text.as_bytes(), markers).map(|offset| line_at(text, offset))
}

fn comment_start(bytes: &[u8], markers: Markers) -> Option<usize> {
    let mut quote: Option<u8> = None;
    let mut index = 0;
    while index < bytes.len() {
        match (quote, bytes[index]) {
            (Some(_), b'\\') => index += 1,
            (Some(open), byte) if byte == open => quote = None,
            (None, byte @ (b'"' | b'\'' | b'`')) => quote = Some(byte),
            (None, b'/') if opens_comment(bytes.get(index + 1), markers) => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
}

fn opens_comment(next: Option<&u8>, markers: Markers) -> bool {
    match next {
        Some(b'*') => true,
        Some(b'/') => matches!(markers, Markers::LineAndBlock),
        _ => false,
    }
}

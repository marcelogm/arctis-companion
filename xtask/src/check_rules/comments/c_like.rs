use crate::text::line_at;

pub fn first_comment_line(text: &str) -> Option<usize> {
    comment_start(text.as_bytes()).map(|offset| line_at(text, offset))
}

fn comment_start(bytes: &[u8]) -> Option<usize> {
    let mut quote: Option<u8> = None;
    let mut index = 0;
    while index < bytes.len() {
        match (quote, bytes[index]) {
            (Some(_), b'\\') => index += 1,
            (Some(open), byte) if byte == open => quote = None,
            (None, byte @ (b'"' | b'\'' | b'`')) => quote = Some(byte),
            (None, b'/') if opens_comment(bytes.get(index + 1)) => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
}

fn opens_comment(next: Option<&u8>) -> bool {
    matches!(next, Some(b'/' | b'*'))
}

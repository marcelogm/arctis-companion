use crate::text::line_at;

pub fn first_comment_line(text: &str) -> Option<usize> {
    text.find("<!--").map(|offset| line_at(text, offset))
}

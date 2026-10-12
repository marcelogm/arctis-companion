use crate::markdown::{outside_code_spans, prose_lines};

pub fn first_comment_line(text: &str) -> Option<usize> {
    prose_lines(text)
        .find(|(_, line)| outside_code_spans(line).any(|plain| plain.contains("<!--")))
        .map(|(number, _)| number)
}

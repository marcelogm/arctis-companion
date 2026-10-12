use crate::rust_tokens::{Token, tokens};
use crate::text::line_at;

pub fn first_comment_line(text: &str) -> Option<usize> {
    tokens(text)
        .find(Token::is_comment)
        .map(|token| line_at(text, token.start))
}

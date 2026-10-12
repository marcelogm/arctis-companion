pub use ra_ap_rustc_lexer::TokenKind;
use ra_ap_rustc_lexer::{FrontmatterAllowed, strip_shebang, tokenize};

pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub start: usize,
}

impl Token<'_> {
    pub fn is_comment(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        )
    }

    pub fn is_ident(&self, word: &str) -> bool {
        self.kind == TokenKind::Ident && self.text == word
    }

    fn is_trivia(&self) -> bool {
        self.is_comment() || self.kind == TokenKind::Whitespace
    }
}

pub fn tokens(source: &str) -> impl Iterator<Item = Token<'_>> {
    let skipped = strip_shebang(source).unwrap_or(0);
    let mut start = skipped;
    tokenize(&source[skipped..], FrontmatterAllowed::No).map(move |token| {
        let end = start + token.len as usize;
        let item = Token {
            kind: token.kind,
            text: &source[start..end],
            start,
        };
        start = end;
        item
    })
}

pub fn code_tokens(source: &str) -> Vec<Token<'_>> {
    tokens(source).filter(|token| !token.is_trivia()).collect()
}

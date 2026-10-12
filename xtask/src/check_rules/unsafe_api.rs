use std::path::Path;

use crate::rust_tokens::{Token, TokenKind, code_tokens};
use crate::text::line_at;

const QUALIFIERS: [&str; 2] = ["const", "async"];

pub fn check(path: &Path, text: &str) -> Option<String> {
    if path.extension().is_none_or(|extension| extension != "rs") {
        return None;
    }
    let tokens = code_tokens(text);
    let position = (0..tokens.len()).find(|&index| is_pub_unsafe_fn(&tokens[index..]))?;
    let line = line_at(text, tokens[position].start);
    Some(format!(
        "{}:{line}: `pub unsafe fn` is not allowed; wrap the unsafe code in a safe function",
        path.display()
    ))
}

fn is_pub_unsafe_fn(tokens: &[Token]) -> bool {
    let Some((first, rest)) = tokens.split_first() else {
        return false;
    };
    first.is_ident("pub") && starts_unsafe_fn(skip_qualifiers(skip_restriction(rest)))
}

fn skip_restriction<'t, 'a>(tokens: &'t [Token<'a>]) -> &'t [Token<'a>] {
    if tokens
        .first()
        .is_none_or(|token| token.kind != TokenKind::OpenParen)
    {
        return tokens;
    }
    let close = tokens
        .iter()
        .position(|token| token.kind == TokenKind::CloseParen);
    &tokens[close.map_or(tokens.len(), |index| index + 1)..]
}

fn skip_qualifiers<'t, 'a>(tokens: &'t [Token<'a>]) -> &'t [Token<'a>] {
    let count = tokens
        .iter()
        .take_while(|token| QUALIFIERS.iter().any(|qualifier| token.is_ident(qualifier)))
        .count();
    &tokens[count..]
}

fn starts_unsafe_fn(tokens: &[Token]) -> bool {
    let Some((first, rest)) = tokens.split_first() else {
        return false;
    };
    first.is_ident("unsafe")
        && skip_abi(rest)
            .first()
            .is_some_and(|token| token.is_ident("fn"))
}

fn skip_abi<'t, 'a>(tokens: &'t [Token<'a>]) -> &'t [Token<'a>] {
    match tokens {
        [keyword, abi, rest @ ..] if keyword.is_ident("extern") && is_literal(abi) => rest,
        [keyword, rest @ ..] if keyword.is_ident("extern") => rest,
        _ => tokens,
    }
}

fn is_literal(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Literal { .. })
}

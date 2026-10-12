use std::ops::Range;
use std::path::Path;

use crate::rust_tokens::{Token, TokenKind, code_tokens};
use crate::text::line_at;

const QUALIFIERS: [&str; 2] = ["const", "async"];
const UNSAFE_FN: &str = "`pub unsafe fn` is not allowed; wrap the unsafe code in a safe function";
const FOREIGN_ITEM: &str =
    "public foreign items are unsafe to use; keep them private or mark them `safe`";

pub fn check(path: &Path, text: &str) -> Option<String> {
    if path.extension().is_none_or(|extension| extension != "rs") {
        return None;
    }
    let tokens = code_tokens(text);
    let (position, problem) = first_problem(&tokens)?;
    let line = line_at(text, tokens[position].start);
    Some(format!("{}:{line}: {problem}", path.display()))
}

fn first_problem(tokens: &[Token]) -> Option<(usize, &'static str)> {
    let unsafe_fn = (0..tokens.len()).find(|&index| is_pub_unsafe_fn(&tokens[index..]));
    let foreign = extern_bodies(tokens).find_map(|body| {
        body.into_iter()
            .find(|&index| is_unmarked_pub(&tokens[index..]))
    });
    unsafe_fn
        .map(|index| (index, UNSAFE_FN))
        .or_else(|| foreign.map(|index| (index, FOREIGN_ITEM)))
}

fn is_pub_unsafe_fn(tokens: &[Token]) -> bool {
    let Some((first, rest)) = tokens.split_first() else {
        return false;
    };
    first.is_ident("pub") && starts_unsafe_fn(skip_qualifiers(skip_restriction(rest)))
}

fn is_unmarked_pub(tokens: &[Token]) -> bool {
    let Some((first, rest)) = tokens.split_first() else {
        return false;
    };
    first.is_ident("pub")
        && skip_restriction(rest)
            .first()
            .is_none_or(|token| !token.is_ident("safe"))
}

fn extern_bodies<'t>(tokens: &'t [Token]) -> impl Iterator<Item = Range<usize>> + 't {
    (0..tokens.len()).filter_map(|index| extern_body(tokens, index))
}

fn extern_body(tokens: &[Token], index: usize) -> Option<Range<usize>> {
    if !tokens[index].is_ident("extern") {
        return None;
    }
    let after_abi = index + 1 + usize::from(tokens.get(index + 1).is_some_and(is_literal));
    let open = tokens.get(after_abi)?;
    (open.kind == TokenKind::OpenBrace).then(|| after_abi + 1..closing_brace(tokens, after_abi))
}

fn closing_brace(tokens: &[Token], open: usize) -> usize {
    let mut depth = 0_usize;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token.kind {
            TokenKind::OpenBrace => depth += 1,
            TokenKind::CloseBrace if depth == 1 => return index,
            TokenKind::CloseBrace => depth -= 1,
            _ => {}
        }
    }
    tokens.len()
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

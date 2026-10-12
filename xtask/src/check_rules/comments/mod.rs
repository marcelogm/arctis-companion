mod c_like;
mod hash;
mod markdown;
mod rust;
mod xml;

use std::path::Path;

use c_like::Markers;

enum Syntax {
    Rust,
    CLike,
    Css,
    Hash,
    Xml,
    Markdown,
    Plain,
}

const HASH_NAMES: [&str; 7] = [
    "meson.build",
    "meson.options",
    "meson_options.txt",
    ".gitignore",
    ".gitattributes",
    "Containerfile",
    "Dockerfile",
];

pub fn check(path: &Path, text: &str) -> Option<String> {
    let Some(syntax) = syntax_of(path, text) else {
        return Some(format!(
            "{}: unknown syntax; add it to comments::syntax_of",
            path.display()
        ));
    };
    let line = first_comment_line(&syntax, text)?;
    Some(format!(
        "{}:{line}: comments are not allowed; rename or extract instead",
        path.display()
    ))
}

fn first_comment_line(syntax: &Syntax, text: &str) -> Option<usize> {
    match syntax {
        Syntax::Rust => rust::first_comment_line(text),
        Syntax::CLike => c_like::first_comment_line(text, Markers::LineAndBlock),
        Syntax::Css => c_like::first_comment_line(text, Markers::BlockOnly),
        Syntax::Hash => hash::first_comment_line(text),
        Syntax::Xml => xml::first_comment_line(text),
        Syntax::Markdown => markdown::first_comment_line(text),
        Syntax::Plain => None,
    }
}

fn syntax_of(path: &Path, text: &str) -> Option<Syntax> {
    let name = path.file_name()?.to_str()?;
    let name = name.strip_suffix(".in").unwrap_or(name);
    if HASH_NAMES.contains(&name) {
        return Some(Syntax::Hash);
    }
    let by_extension = name.rsplit_once('.').and_then(|(_, ext)| by_extension(ext));
    by_extension.or_else(|| is_script(text).then_some(Syntax::Hash))
}

fn is_script(text: &str) -> bool {
    text.starts_with("#!/") || text.starts_with("#! /")
}

fn by_extension(extension: &str) -> Option<Syntax> {
    match extension {
        "rs" => Some(Syntax::Rust),
        "js" | "mjs" | "cjs" | "ts" | "blp" | "scss" => Some(Syntax::CLike),
        "css" => Some(Syntax::Css),
        "toml" | "lock" | "yml" | "yaml" | "sh" | "py" | "ini" | "conf" | "desktop" | "service"
        | "rules" | "po" | "pot" => Some(Syntax::Hash),
        "xml" | "ui" | "svg" | "policy" => Some(Syntax::Xml),
        "md" => Some(Syntax::Markdown),
        "json" | "txt" | "hex" => Some(Syntax::Plain),
        _ => None,
    }
}

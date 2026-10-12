mod c_like;
mod hash;
mod rust;
mod xml;

use std::path::Path;

enum Syntax {
    Rust,
    CLike,
    Hash,
    Xml,
}

const HASH_NAMES: [&str; 5] = [
    "meson.build",
    "meson.options",
    "meson_options.txt",
    ".gitignore",
    ".gitattributes",
];

pub fn check(path: &Path, text: &str) -> Option<String> {
    let line = match syntax_of(path)? {
        Syntax::Rust => rust::first_comment_line(text),
        Syntax::CLike => c_like::first_comment_line(text),
        Syntax::Hash => hash::first_comment_line(text),
        Syntax::Xml => xml::first_comment_line(text),
    }?;
    Some(format!(
        "{}:{line}: comments are not allowed; rename or extract instead",
        path.display()
    ))
}

fn syntax_of(path: &Path) -> Option<Syntax> {
    let name = path.file_name()?.to_str()?;
    let name = name.strip_suffix(".in").unwrap_or(name);
    if HASH_NAMES.contains(&name) {
        return Some(Syntax::Hash);
    }
    by_extension(name.rsplit_once('.')?.1)
}

fn by_extension(extension: &str) -> Option<Syntax> {
    match extension {
        "rs" => Some(Syntax::Rust),
        "js" | "mjs" | "blp" | "css" => Some(Syntax::CLike),
        "toml" | "yml" | "yaml" | "sh" | "desktop" | "service" | "rules" => Some(Syntax::Hash),
        "xml" => Some(Syntax::Xml),
        _ => None,
    }
}

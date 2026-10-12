use std::fs;
use std::path::{Path, PathBuf};

use crate::doc_refs::{DocRef, doc_refs};
use crate::report::Report;
use crate::rust_tokens::code_tokens;

const DECLARATIONS: [&str; 4] = ["fn", "struct", "enum", "trait"];

const CODE_LINE: &str = "- Code:";

#[derive(Clone, Copy)]
enum Scope {
    Everything,
    CodeLines,
}

pub fn run(root: &Path) -> Report {
    documents(root)
        .iter()
        .map(|(document, scope)| check_document(root, document, *scope))
        .fold(Report::default(), Report::merged)
}

fn documents(root: &Path) -> Vec<(PathBuf, Scope)> {
    let mut documents: Vec<(PathBuf, Scope)> = markdown_under(&root.join("docs/architecture"))
        .into_iter()
        .map(|path| (path, Scope::Everything))
        .collect();
    documents.push((root.join("CONTEXT.md"), Scope::Everything));
    documents.extend(adrs(root).into_iter().map(|path| (path, Scope::CodeLines)));
    documents.retain(|(document, _)| document.is_file());
    documents
}

fn adrs(root: &Path) -> Vec<PathBuf> {
    markdown_under(&root.join("docs/adr"))
        .into_iter()
        .filter(|path| is_numbered(path))
        .collect()
}

fn is_numbered(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(|first: char| first.is_ascii_digit()))
}

fn markdown_under(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .flat_map(expand)
        .collect()
}

fn expand(path: PathBuf) -> Vec<PathBuf> {
    if path.is_dir() {
        return markdown_under(&path);
    }
    if path.extension().is_some_and(|extension| extension == "md") {
        vec![path]
    } else {
        Vec::new()
    }
}

fn check_document(root: &Path, document: &Path, scope: Scope) -> Report {
    let label = document
        .strip_prefix(root)
        .unwrap_or(document)
        .display()
        .to_string();
    let Ok(text) = fs::read_to_string(document) else {
        return Report::failed(format!("{label}: unreadable"));
    };
    let lines: Vec<&str> = text.lines().collect();
    let problems = doc_refs(&text)
        .into_iter()
        .filter(|doc_ref| in_scope(scope, &lines, doc_ref.line))
        .filter_map(|doc_ref| {
            problem(root, &doc_ref).map(|problem| format!("{label}:{}: {problem}", doc_ref.line))
        });
    Report::from_errors(problems)
}

fn in_scope(scope: Scope, lines: &[&str], line: usize) -> bool {
    match scope {
        Scope::Everything => true,
        Scope::CodeLines => line
            .checked_sub(1)
            .and_then(|index| lines.get(index))
            .is_some_and(|text| text.trim_start().starts_with(CODE_LINE)),
    }
}

fn problem(root: &Path, doc_ref: &DocRef) -> Option<String> {
    let target = root.join(&doc_ref.path);
    let wants_directory = doc_ref.path.ends_with('/');
    if !target.exists() || (wants_directory && !target.is_dir()) {
        return Some(format!("`{}` does not exist", doc_ref.path));
    }
    let item = doc_ref.item.as_deref()?;
    (!declares(&target, item)).then(|| {
        format!(
            "`{}` declares no fn, struct, enum or trait `{item}`",
            doc_ref.path
        )
    })
}

fn declares(file: &Path, item: &str) -> bool {
    let text = fs::read_to_string(file).unwrap_or_default();
    let tokens = code_tokens(&text);
    tokens.windows(2).any(|pair| {
        DECLARATIONS.iter().any(|keyword| pair[0].is_ident(keyword)) && pair[1].is_ident(item)
    })
}

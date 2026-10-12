use std::fs;
use std::path::{Path, PathBuf};

use crate::doc_refs::{DocRef, doc_refs};
use crate::report::Report;
use crate::rust_tokens::code_tokens;

const DECLARATIONS: [&str; 4] = ["fn", "struct", "enum", "trait"];

pub fn run(root: &Path) -> Report {
    documents(root)
        .iter()
        .map(|document| check_document(root, document))
        .fold(Report::default(), Report::merged)
}

fn documents(root: &Path) -> Vec<PathBuf> {
    let mut documents = markdown_under(&root.join("docs/architecture"));
    documents.push(root.join("CONTEXT.md"));
    documents.retain(|document| document.is_file());
    documents
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

fn check_document(root: &Path, document: &Path) -> Report {
    let label = document
        .strip_prefix(root)
        .unwrap_or(document)
        .display()
        .to_string();
    let Ok(text) = fs::read_to_string(document) else {
        return Report::failed(format!("{label}: unreadable"));
    };
    let problems = doc_refs(&text).into_iter().filter_map(|doc_ref| {
        problem(root, &doc_ref).map(|problem| format!("{label}:{}: {problem}", doc_ref.line))
    });
    Report::errors(problems)
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

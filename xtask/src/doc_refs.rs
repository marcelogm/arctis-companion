pub struct DocRef {
    pub line: usize,
    pub path: String,
    pub item: Option<String>,
}

const ROOT_EXTENSIONS: [&str; 9] = [
    "rs", "md", "toml", "json", "xml", "js", "blp", "build", "lock",
];
const NOT_A_PATH: [&str; 8] = ["://", "//", "*", "{", "<", "\u{2026}", "...", "$"];

pub fn doc_refs(markdown: &str) -> Vec<DocRef> {
    prose_lines(markdown)
        .flat_map(|(line, text)| spans(text).filter_map(move |span| as_doc_ref(line, span)))
        .collect()
}

fn prose_lines(markdown: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut in_fence = false;
    markdown
        .lines()
        .enumerate()
        .filter(move |(_, text)| {
            let fence = text.trim_start().starts_with("```");
            in_fence ^= fence;
            !fence && !in_fence
        })
        .map(|(index, text)| (index + 1, text))
}

fn spans(text: &str) -> impl Iterator<Item = &str> {
    text.split('`').skip(1).step_by(2)
}

fn as_doc_ref(line: usize, span: &str) -> Option<DocRef> {
    let (path, item) = split_item(span);
    looks_like_path(path).then(|| DocRef {
        line,
        path: path.to_owned(),
        item,
    })
}

fn split_item(span: &str) -> (&str, Option<String>) {
    match span.split_once(".rs::") {
        Some((stem, items)) => {
            let path = &span[..stem.len() + ".rs".len()];
            (path, items.rsplit("::").next().map(str::to_owned))
        }
        None => (span, None),
    }
}

fn looks_like_path(span: &str) -> bool {
    let plain = !span.is_empty()
        && !span.contains(char::is_whitespace)
        && !span.starts_with(['/', '~', '-'])
        && !NOT_A_PATH.iter().any(|marker| span.contains(marker));
    plain && (span.contains('/') || has_root_extension(span))
}

fn has_root_extension(span: &str) -> bool {
    span.rsplit_once('.')
        .is_some_and(|(stem, extension)| !stem.is_empty() && ROOT_EXTENSIONS.contains(&extension))
}

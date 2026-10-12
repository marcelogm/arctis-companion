const MARKERS: [&str; 5] = [
    "Plain(\"",
    "msgid: \"",
    "singular: \"",
    "plural: \"",
    "Msgid(\"",
];

pub fn msgids_in(dump: &str) -> Vec<String> {
    MARKERS
        .iter()
        .flat_map(|marker| {
            dump.match_indices(marker)
                .map(move |(at, _)| debug_string(&dump[at + marker.len()..]))
        })
        .collect()
}

fn debug_string(rest: &str) -> String {
    let mut text = String::new();
    let mut chars = rest.chars();
    while let Some(next) = chars.next() {
        match next {
            '"' => break,
            '\\' => text.push(unescaped(chars.next())),
            other => text.push(other),
        }
    }
    text
}

const fn unescaped(escaped: Option<char>) -> char {
    match escaped {
        Some('n') => '\n',
        Some('t') => '\t',
        Some(other) => other,
        None => '\\',
    }
}

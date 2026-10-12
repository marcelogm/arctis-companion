#[must_use]
pub fn name_from_json(value: &str) -> Option<String> {
    let after_key = value.split_once("\"name\"")?.1;
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let quoted = after_colon.trim_start().strip_prefix('"')?;
    unescape_until_quote(quoted)
}

fn unescape_until_quote(text: &str) -> Option<String> {
    let mut name = String::new();
    let mut chars = text.chars();
    while let Some(next) = chars.next() {
        match next {
            '"' => return Some(name),
            '\\' => name.push(chars.next()?),
            _ => name.push(next),
        }
    }
    None
}

#[must_use]
pub fn json_for_name(name: &str) -> String {
    let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"name\":\"{escaped}\"}}")
}

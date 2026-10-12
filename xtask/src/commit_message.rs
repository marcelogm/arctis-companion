const TYPES: [&str; 11] = [
    "feat", "fix", "refactor", "test", "docs", "build", "ci", "chore", "perf", "style", "revert",
];
const FORBIDDEN: [&str; 8] = [
    "co-authored-by",
    "generated with",
    "claude",
    "anthropic",
    "openai",
    "chatgpt",
    "copilot",
    "\u{1f916}",
];

pub fn subject_problem(subject: &str) -> Option<String> {
    (!is_conventional(subject)).then(|| {
        format!(
            "subject \"{subject}\" is not `type(scope)!: summary` with type one of {}",
            TYPES.join(", ")
        )
    })
}

pub fn attribution_problem(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    FORBIDDEN
        .iter()
        .find(|word| lower.contains(*word))
        .map(|word| format!("mentions \"{word}\"; attribution and tool mentions are not allowed"))
}

pub fn editor_message(raw: &str) -> String {
    let kept: Vec<&str> = raw
        .lines()
        .take_while(|line| !is_scissors(line))
        .filter(|line| !line.starts_with('#'))
        .collect();
    kept.join("\n")
}

pub fn subject_of(message: &str) -> &str {
    message
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
}

fn is_scissors(line: &str) -> bool {
    line.starts_with('#') && line.contains(">8")
}

fn is_conventional(subject: &str) -> bool {
    let Some((head, summary)) = subject.split_once(": ") else {
        return false;
    };
    let head = head.strip_suffix('!').unwrap_or(head);
    !summary.trim().is_empty() && head_is_valid(head)
}

fn head_is_valid(head: &str) -> bool {
    match head.split_once('(') {
        Some((kind, scope)) => TYPES.contains(&kind) && scope_is_valid(scope),
        None => TYPES.contains(&head),
    }
}

fn scope_is_valid(scope: &str) -> bool {
    scope
        .strip_suffix(')')
        .is_some_and(|name| !name.is_empty() && name.chars().all(is_scope_char))
}

fn is_scope_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || "-_/.".contains(character)
}

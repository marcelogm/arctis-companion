pub fn first_comment_line(text: &str) -> Option<usize> {
    text.lines()
        .enumerate()
        .find(|(index, line)| has_comment(line, *index == 0))
        .map(|(index, _)| index + 1)
}

fn has_comment(line: &str, first_line: bool) -> bool {
    if first_line && line.starts_with("#!") {
        return false;
    }
    let mut quote = None;
    let mut previous = ' ';
    for character in line.chars() {
        if quote.is_none() && character == '#' && previous.is_whitespace() {
            return true;
        }
        quote = next_quote(quote, previous, character);
        previous = character;
    }
    false
}

fn next_quote(quote: Option<char>, previous: char, character: char) -> Option<char> {
    match quote {
        Some(open) if character == open && previous != '\\' => None,
        Some(open) => Some(open),
        None if character == '"' || character == '\'' => Some(character),
        None => None,
    }
}

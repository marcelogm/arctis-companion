#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Code,
    Double,
    DoubleEscape,
    Single,
}

impl State {
    fn next(self, character: char) -> Self {
        match (self, character) {
            (Self::Code, '"') | (Self::DoubleEscape, _) => Self::Double,
            (Self::Code, '\'') => Self::Single,
            (Self::Double, '\\') => Self::DoubleEscape,
            (Self::Double, '"') | (Self::Single, '\'') => Self::Code,
            (state, _) => state,
        }
    }
}

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
    let mut state = State::Code;
    let mut previous = ' ';
    for character in line.chars() {
        if state == State::Code && character == '#' && previous.is_whitespace() {
            return true;
        }
        state = state.next(character);
        previous = character;
    }
    false
}

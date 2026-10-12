#[derive(Default)]
struct Fences {
    open: bool,
}

impl Fences {
    fn is_prose(&mut self, line: &str) -> bool {
        if line.trim_start().starts_with("```") {
            self.open = !self.open;
            return false;
        }
        !self.open
    }
}

pub fn prose_lines(markdown: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut fences = Fences::default();
    markdown
        .lines()
        .enumerate()
        .filter(move |(_, line)| fences.is_prose(line))
        .map(|(index, line)| (index + 1, line))
}

pub fn code_spans(line: &str) -> impl Iterator<Item = &str> {
    line.split('`').skip(1).step_by(2)
}

pub fn outside_code_spans(line: &str) -> impl Iterator<Item = &str> {
    line.split('`').step_by(2)
}

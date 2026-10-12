use std::process::ExitCode;

#[derive(Default)]
pub struct Report {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl Report {
    pub fn failed(message: impl Into<String>) -> Self {
        Self::from_errors([message.into()])
    }

    pub fn from_errors(messages: impl IntoIterator<Item = String>) -> Self {
        Self {
            errors: messages.into_iter().collect(),
            warnings: Vec::new(),
        }
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    pub fn merged(mut self, other: Self) -> Self {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self
    }

    pub fn finish(self) -> ExitCode {
        self.warnings
            .iter()
            .for_each(|warning| eprintln!("warning: {warning}"));
        self.errors
            .iter()
            .for_each(|error| eprintln!("error: {error}"));
        if self.errors.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }
}

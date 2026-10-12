#[must_use]
pub const fn msgid(english: &'static str) -> &'static str {
    english
}

#[must_use]
pub const fn forms(singular: &'static str, plural: &'static str) -> [&'static str; 2] {
    [singular, plural]
}

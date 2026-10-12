use arctis_core::battery::sample::UnixSeconds;
use arctis_core::view::text::{Arg, Text};
use gettextrs::{gettext, ngettext};
use gtk::glib;

const DECIMAL_COMMA_LANGUAGES: [&str; 31] = [
    "bg", "ca", "cs", "da", "de", "el", "es", "et", "fi", "fr", "gl", "hr", "hu", "id", "it", "lt",
    "lv", "nb", "nl", "nn", "pl", "pt", "ro", "ru", "sk", "sl", "sr", "sv", "tr", "uk", "vi",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberStyle {
    decimal: char,
}

impl NumberStyle {
    pub const POINT: Self = Self { decimal: '.' };
    const COMMA: Self = Self { decimal: ',' };

    #[must_use]
    pub fn for_locale(locale: &str) -> Self {
        let language = locale.split(['_', '.', '@']).next().unwrap_or_default();
        if DECIMAL_COMMA_LANGUAGES.contains(&language) {
            Self::COMMA
        } else {
            Self::POINT
        }
    }

    #[must_use]
    pub fn current() -> Self {
        glib::language_names_with_category("LC_NUMERIC")
            .first()
            .map_or(Self::POINT, |name| Self::for_locale(name))
    }

    fn signed_tenths(self, value: f64) -> String {
        format!("{value:+.1}").replace('.', &self.decimal.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolver {
    numbers: NumberStyle,
}

impl Resolver {
    #[must_use]
    pub const fn new(numbers: NumberStyle) -> Self {
        Self { numbers }
    }

    #[must_use]
    pub fn current() -> Self {
        Self::new(NumberStyle::current())
    }

    #[must_use]
    pub fn resolve(&self, text: &Text) -> String {
        match text {
            Text::Verbatim(text) => text.clone(),
            Text::Plain(msgid) => gettext(msgid.as_str()),
            Text::Format { msgid, args } => self.fill(&gettext(msgid.as_str()), args),
            Text::Plural {
                singular,
                plural,
                n,
                args,
            } => {
                let form = ngettext(singular.as_str(), plural.as_str(), *n);
                self.fill(&form.replace("{n}", &n.to_string()), args)
            }
        }
    }

    fn fill(&self, template: &str, args: &[(&str, Arg)]) -> String {
        args.iter().fold(template.to_owned(), |text, (name, arg)| {
            text.replace(&format!("{{{name}}}"), &self.argument(arg))
        })
    }

    fn argument(&self, arg: &Arg) -> String {
        match arg {
            Arg::Percent(percent) => format!("{} %", percent.get()),
            Arg::Decibels(gain) => format!("{} dB", self.numbers.signed_tenths(gain.db())),
            Arg::Hertz(hertz) => format!("{} Hz", hertz.get()),
            Arg::Number(number) => number.to_string(),
            Arg::Time(at) => local_time(*at),
            Arg::Msgid(msgid) => gettext(msgid.as_str()),
            Arg::Str(text) => text.clone(),
        }
    }
}

#[must_use]
pub fn resolve(text: &Text) -> String {
    Resolver::current().resolve(text)
}

#[must_use]
pub fn optional(text: Option<&Text>) -> String {
    text.map(resolve).unwrap_or_default()
}

fn local_time(at: UnixSeconds) -> String {
    i64::try_from(at.0)
        .ok()
        .and_then(|seconds| glib::DateTime::from_unix_local(seconds).ok())
        .and_then(|time| time.format("%x %R").ok())
        .map(String::from)
        .unwrap_or_default()
}

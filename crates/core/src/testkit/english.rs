use crate::view::text::{Arg, Text};

#[must_use]
pub fn english(text: &Text) -> String {
    match text {
        Text::Verbatim(text) => text.clone(),
        Text::Plain(msgid) => substitute(msgid, &[]),
        Text::Format { msgid, args } => substitute(msgid, args),
        Text::Plural {
            singular,
            plural,
            n,
            args,
        } => {
            let form = if *n == 1 { singular } else { plural };
            substitute(&form.replace("{n}", &n.to_string()), args)
        }
    }
}

fn substitute(template: &str, args: &[(&str, Arg)]) -> String {
    let unbound = args.iter().fold(template.to_owned(), |text, (name, _)| {
        text.replace(&placeholder(name), "")
    });
    assert!(
        !unbound.contains('{'),
        "unbound placeholder in {template:?}"
    );
    args.iter().fold(template.to_owned(), |text, (name, arg)| {
        text.replace(&placeholder(name), &argument(arg))
    })
}

fn placeholder(name: &str) -> String {
    format!("{{{name}}}")
}

fn argument(arg: &Arg) -> String {
    match arg {
        Arg::Percent(percent) => format!("{} %", percent.get()),
        Arg::Decibels(gain) => format!("{:+.1} dB", gain.db()),
        Arg::Hertz(hertz) => format!("{} Hz", hertz.get()),
        Arg::Number(number) => number.to_string(),
        Arg::Time(at) => format!("@{}", at.0),
        Arg::Msgid(text) | Arg::Str(text) => text.clone(),
    }
}

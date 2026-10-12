const SCALE: i64 = 1_000;
const DIGITS_READ: usize = 4;

#[must_use]
pub fn parse_thousandths(text: &str) -> Option<i64> {
    match text.strip_prefix('-') {
        Some(magnitude) => unsigned_thousandths(magnitude).map(|value| -value),
        None => unsigned_thousandths(text.strip_prefix('+').unwrap_or(text)),
    }
}

#[must_use]
pub fn thousandths_of(value: f64) -> Option<i64> {
    parse_thousandths(&value.to_string())
}

#[must_use]
pub fn render_thousandths(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let magnitude = value.unsigned_abs();
    let scale = SCALE.unsigned_abs();
    let (whole, fraction) = (magnitude / scale, magnitude % scale);
    if fraction == 0 {
        return format!("{sign}{whole}");
    }
    let digits = format!("{fraction:03}");
    format!("{sign}{whole}.{}", digits.trim_end_matches('0'))
}

#[must_use]
pub fn rounded_quotient(value: i64, divisor: i64) -> i64 {
    (value + value.signum() * (divisor / 2)) / divisor
}

fn unsigned_thousandths(text: &str) -> Option<i64> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty() || !all_digits(whole) || !all_digits(fraction) {
        return None;
    }
    let whole: i64 = whole.parse().ok()?;
    whole
        .checked_mul(SCALE)?
        .checked_add(fraction_thousandths(fraction))
}

fn all_digits(text: &str) -> bool {
    text.bytes().all(|byte| byte.is_ascii_digit())
}

fn fraction_thousandths(fraction: &str) -> i64 {
    let padded = fraction
        .bytes()
        .chain(std::iter::repeat(b'0'))
        .take(DIGITS_READ);
    let ten_thousandths = padded.fold(0, |total, digit| total * 10 + i64::from(digit - b'0'));
    (ten_thousandths + 5) / 10
}

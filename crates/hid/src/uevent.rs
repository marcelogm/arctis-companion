#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Uevent {
    pub vendor: u16,
    pub product: u16,
    pub physical: String,
    pub interface: u8,
    pub uniq: Option<String>,
}

pub fn parse(text: &str) -> Option<Uevent> {
    let (vendor, product) = hid_id(value_of(text, "HID_ID")?)?;
    let (physical, interface) = hid_phys(value_of(text, "HID_PHYS")?)?;
    Some(Uevent {
        vendor,
        product,
        physical: physical.to_owned(),
        interface,
        uniq: value_of(text, "HID_UNIQ")
            .filter(|uniq| !uniq.is_empty())
            .map(str::to_owned),
    })
}

fn value_of<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value.trim())
}

fn hid_id(value: &str) -> Option<(u16, u16)> {
    let mut parts = value.split(':').skip(1);
    let vendor = hex_u16(parts.next()?)?;
    let product = hex_u16(parts.next()?)?;
    Some((vendor, product))
}

fn hex_u16(text: &str) -> Option<u16> {
    u32::from_str_radix(text, 16)
        .ok()
        .and_then(|value| u16::try_from(value).ok())
}

fn hid_phys(value: &str) -> Option<(&str, u8)> {
    let (physical, interface) = value.rsplit_once("/input")?;
    Some((physical, interface.parse().ok()?))
}

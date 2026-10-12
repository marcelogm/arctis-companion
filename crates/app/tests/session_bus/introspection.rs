pub fn canonical_members(xml: &str, interface: &str) -> Vec<String> {
    let start = xml
        .find(&format!("<interface name=\"{interface}\""))
        .expect("the interface is present");
    let body = &xml[start..];
    let end = body.find("</interface>").expect("the interface is closed");
    let mut members: Vec<String> = body[..end].split('<').filter_map(canonical_tag).collect();
    members.sort();
    members
}

fn canonical_tag(tag: &str) -> Option<String> {
    let tag = tag.trim().trim_end_matches('>').trim_end_matches('/');
    let mut parts = tag.split_whitespace();
    let kind = parts.next()?;
    if !["property", "method", "arg", "signal"].contains(&kind) {
        return None;
    }
    let mut attributes: Vec<&str> = parts.collect();
    attributes.sort_unstable();
    Some(format!("{kind} {}", attributes.join(" ")))
}

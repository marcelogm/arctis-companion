use super::unescape;

const MARKERS: [&str; 5] = [
    "Plain(\"",
    "msgid: \"",
    "singular: \"",
    "plural: \"",
    "Msgid(\"",
];

pub fn msgids_in(dump: &str) -> Vec<String> {
    MARKERS
        .iter()
        .flat_map(|marker| {
            dump.match_indices(marker)
                .map(move |(at, _)| unescape(&dump[at + marker.len()..]))
        })
        .collect()
}

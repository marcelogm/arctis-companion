use std::collections::HashMap;

use super::unescape;

pub type Group = (Vec<String>, Vec<String>);

pub struct Catalogue {
    translations: HashMap<String, Vec<String>>,
    groups: Vec<Group>,
}

impl Catalogue {
    pub fn parse(text: &str) -> Self {
        let entries: Vec<Entry> = text
            .split("\n\n")
            .map(Entry::parse)
            .filter(|entry| !entry.msgids.is_empty() && !entry.fuzzy)
            .collect();
        let groups = entries
            .iter()
            .map(|entry| (entry.msgids.clone(), entry.msgstrs.clone()))
            .collect();
        let translations = entries.into_iter().flat_map(Entry::keyed).collect();
        Self {
            translations,
            groups,
        }
    }

    pub fn is_translated(&self, msgid: &str) -> bool {
        self.translations
            .get(msgid)
            .is_some_and(|forms| all_filled(forms))
    }

    pub fn translated_groups(&self) -> impl Iterator<Item = &Group> {
        self.groups.iter().filter(|(_, forms)| all_filled(forms))
    }
}

fn all_filled(forms: &[String]) -> bool {
    !forms.is_empty() && forms.iter().all(|form| !form.is_empty())
}

#[derive(Default)]
struct Entry {
    msgids: Vec<String>,
    msgstrs: Vec<String>,
    fuzzy: bool,
}

#[derive(Clone, Copy)]
enum Field {
    Msgid,
    Msgstr,
}

impl Entry {
    fn parse(block: &str) -> Self {
        let mut entry = Self::default();
        let mut target: Option<Field> = None;
        for line in block.lines() {
            target = entry.take(line, target);
        }
        entry
    }

    fn take(&mut self, line: &str, target: Option<Field>) -> Option<Field> {
        if line.starts_with("#,") {
            self.fuzzy |= line.contains("fuzzy");
            return target;
        }
        if let Some(rest) = line.strip_prefix('"') {
            self.append(target, &unescape(rest));
            return target;
        }
        let (keyword, value) = line.split_once(' ')?;
        if keyword == "msgctxt" || !keyword.starts_with("msg") {
            return None;
        }
        let field = if keyword.starts_with("msgstr") {
            Field::Msgstr
        } else {
            Field::Msgid
        };
        self.field(field)
            .push(unescape(value.trim_start_matches('"')));
        Some(field)
    }

    fn append(&mut self, target: Option<Field>, text: &str) {
        let Some(field) = target else {
            return;
        };
        if let Some(last) = self.field(field).last_mut() {
            last.push_str(text);
        }
    }

    fn field(&mut self, field: Field) -> &mut Vec<String> {
        match field {
            Field::Msgid => &mut self.msgids,
            Field::Msgstr => &mut self.msgstrs,
        }
    }

    fn keyed(self) -> Vec<(String, Vec<String>)> {
        self.msgids
            .into_iter()
            .map(|msgid| (msgid, self.msgstrs.clone()))
            .collect()
    }
}

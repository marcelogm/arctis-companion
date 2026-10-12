use std::collections::HashMap;

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

impl Entry {
    fn parse(block: &str) -> Self {
        let mut entry = Self::default();
        let mut target: Option<bool> = None;
        for line in block.lines() {
            target = entry.take(line, target);
        }
        entry
    }

    fn take(&mut self, line: &str, target: Option<bool>) -> Option<bool> {
        if line.starts_with("#,") {
            self.fuzzy |= line.contains("fuzzy");
            return target;
        }
        if let Some(rest) = line.strip_prefix('"') {
            self.append(target, &unquoted(rest));
            return target;
        }
        let (keyword, value) = line.split_once(' ')?;
        if keyword == "msgctxt" {
            return None;
        }
        let is_msgstr = keyword.starts_with("msgstr");
        let field = if is_msgstr {
            &mut self.msgstrs
        } else {
            &mut self.msgids
        };
        keyword.starts_with("msg").then(|| {
            field.push(unquoted(value.trim_start_matches('"')));
            is_msgstr
        })
    }

    fn append(&mut self, target: Option<bool>, text: &str) {
        let field = match target {
            Some(true) => &mut self.msgstrs,
            Some(false) => &mut self.msgids,
            None => return,
        };
        if let Some(last) = field.last_mut() {
            last.push_str(text);
        }
    }

    fn keyed(self) -> Vec<(String, Vec<String>)> {
        self.msgids
            .into_iter()
            .map(|msgid| (msgid, self.msgstrs.clone()))
            .collect()
    }
}

fn unquoted(quoted: &str) -> String {
    let body = quoted.strip_suffix('"').unwrap_or(quoted);
    body.replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

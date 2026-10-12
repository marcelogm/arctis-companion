use super::text::Text;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusPageModel {
    pub icon: &'static str,
    pub title: Text,
    pub description: Text,
    pub button: Option<Text>,
}

impl StatusPageModel {
    #[must_use]
    pub fn new(icon: &'static str, title: &str, description: &str, button: Option<&str>) -> Self {
        Self {
            icon,
            title: Text::plain(title),
            description: Text::plain(description),
            button: button.map(Text::plain),
        }
    }
}

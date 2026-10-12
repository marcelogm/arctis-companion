use std::fs;
use std::path::PathBuf;

use arctis_ui::app_icons::{FALLBACK_ICON, icon_name};

use crate::support::init;

const EMPTY_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"/>"#;

fn icon_dir(icon: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arctis-icons-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("icon directory");
    fs::write(dir.join(format!("{icon}.svg")), EMPTY_SVG).expect("icon file");
    dir
}

fn theme_with(icon: &str) -> gtk::IconTheme {
    let dir = icon_dir(icon);
    let theme = gtk::IconTheme::new();
    theme.set_search_path(&[dir.as_path()]);
    theme
}

#[gtk::test]
fn an_identifier_the_theme_knows_is_its_own_icon() {
    init();
    let theme = theme_with("org.example.Player");
    assert_eq!(
        icon_name(&theme, "org.example.Player"),
        "org.example.Player"
    );
}

#[gtk::test]
fn an_unknown_identifier_gets_the_generic_icon() {
    init();
    let theme = theme_with("org.example.Player");
    assert_eq!(icon_name(&theme, "org.example.NoSuchPlayer"), FALLBACK_ICON);
}

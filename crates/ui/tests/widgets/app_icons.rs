use std::fs;
use std::path::PathBuf;

use arctis_ui::app_icons::{FALLBACK_ICON, icon_name};

use crate::support::init;

const EMPTY_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"/>"#;

fn icon_dir(test: &str, icon: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("icons")
        .join(test);
    fs::create_dir_all(&dir).expect("icon directory");
    fs::write(dir.join(format!("{icon}.svg")), EMPTY_SVG).expect("icon file");
    dir
}

fn theme_with(test: &str, icon: &str) -> gtk::IconTheme {
    let dir = icon_dir(test, icon);
    let theme = gtk::IconTheme::new();
    theme.set_search_path(&[dir.as_path()]);
    theme
}

#[gtk::test]
fn an_identifier_the_theme_knows_is_its_own_icon() {
    init();
    let theme = theme_with("known", "org.example.Player");
    assert_eq!(
        icon_name(&theme, "org.example.Player"),
        "org.example.Player"
    );
}

#[gtk::test]
fn an_unknown_identifier_gets_the_generic_icon() {
    init();
    let theme = theme_with("unknown", "org.example.Player");
    assert_eq!(icon_name(&theme, "org.example.NoSuchPlayer"), FALLBACK_ICON);
}

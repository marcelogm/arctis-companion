use std::path::Path;

use arctis_app::locale::{GETTEXT_PACKAGE, locale_dir_beside};

#[test]
fn the_catalogue_sits_under_the_prefix_share_dir() {
    assert_eq!(
        locale_dir_beside(Path::new("/app/bin/arctis-companion")).as_deref(),
        Some(Path::new("/app/share/locale"))
    );
}

#[test]
fn a_bare_name_has_no_prefix() {
    assert_eq!(locale_dir_beside(Path::new("arctis-companion")), None);
}

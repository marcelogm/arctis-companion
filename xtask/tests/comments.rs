mod support;

use support::{Fixture, assert_accepted, assert_rejected};

#[test]
fn staged_mode_rejects_a_staged_comment() {
    let fixture = Fixture::init();
    fixture.write("src/lib.rs", "// why\n");
    assert!(!fixture.xtask(&["check-rules", "--staged"]).succeeded());
}

#[test]
fn rust_outer_doc_comment_is_rejected() {
    assert_rejected("src/lib.rs", "/// Adds two numbers.\npub fn add() {}\n");
}

#[test]
fn rust_inner_doc_comment_is_rejected() {
    assert_rejected("src/lib.rs", "//! The crate.\n");
}

#[test]
fn rust_line_comment_is_rejected() {
    assert_rejected("src/lib.rs", "fn f() {} // why\n");
}

#[test]
fn rust_block_comment_is_rejected() {
    assert_rejected("src/lib.rs", "fn f() { /* why */ }\n");
}

#[test]
fn rust_comment_markers_inside_strings_are_accepted() {
    assert_accepted(
        "src/lib.rs",
        "pub const URL: &str = \"https://example.org/*x*/\";\n",
    );
}

#[test]
fn rust_inner_attribute_is_accepted() {
    assert_accepted("src/main.rs", "#![forbid(unsafe_code)]\nfn main() {}\n");
}

#[test]
fn javascript_line_comment_is_rejected() {
    assert_rejected("extension/lib/client.js", "export const a = 1; // why\n");
}

#[test]
fn javascript_block_comment_is_rejected() {
    assert_rejected(
        "extension/lib/client.js",
        "/* why */\nexport const a = 1;\n",
    );
}

#[test]
fn javascript_url_inside_a_string_is_accepted() {
    assert_accepted(
        "extension/lib/client.js",
        "export const url = 'https://example.org';\n",
    );
}

#[test]
fn toml_comment_is_rejected() {
    assert_rejected("config/settings.toml", "# why\nvalue = 1\n");
}

#[test]
fn toml_trailing_comment_is_rejected() {
    assert_rejected("config/settings.toml", "value = 1 # why\n");
}

#[test]
fn toml_hash_inside_a_string_is_accepted() {
    assert_accepted("config/settings.toml", "color = \"#ffffff\"\n");
}

#[test]
fn blueprint_comment_is_rejected() {
    assert_rejected("crates/ui/resources/window.blp", "using Gtk 4.0;\n// why\n");
}

#[test]
fn css_comment_is_rejected() {
    assert_rejected("crates/ui/resources/style.css", "/* why */\n.row {}\n");
}

#[test]
fn xml_comment_is_rejected() {
    assert_rejected("data/contract.xml", "<node><!-- why --></node>\n");
}

#[test]
fn yaml_comment_is_rejected() {
    assert_rejected(".github/workflows/ci.yml", "on: push # why\n");
}

#[test]
fn shell_shebang_is_accepted() {
    assert_accepted("build-aux/run.sh", "#!/bin/sh\necho \"#1\"\n");
}

#[test]
fn shell_comment_is_rejected() {
    assert_rejected("build-aux/run.sh", "#!/bin/sh\n# why\n");
}

#[test]
fn meson_comment_is_rejected() {
    assert_rejected("meson.build", "project('a') # why\n");
}

#[test]
fn desktop_entry_comment_is_rejected() {
    assert_rejected("data/app.desktop", "# why\n[Desktop Entry]\n");
}

#[test]
fn template_file_uses_the_syntax_of_its_output() {
    assert_rejected("data/app.service.in", "# why\n[D-BUS Service]\n");
}

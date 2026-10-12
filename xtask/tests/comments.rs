mod support;

use support::{Fixture, assert_accepted, assert_rejected};

#[test]
fn staged_mode_rejects_a_staged_comment() {
    let fixture = Fixture::init();
    fixture.write("src/lib.rs", "// why\n");
    assert!(!fixture.xtask(&["check-rules", "--staged"]).succeeded());
}

#[test]
fn staged_mode_checks_the_index_not_the_working_tree() {
    let fixture = Fixture::init();
    fixture
        .write("src/lib.rs", "fn f() {} // why\n")
        .edit_unstaged("src/lib.rs", "fn f() {}\n");
    assert!(!fixture.xtask(&["check-rules", "--staged"]).succeeded());
}

#[test]
fn staged_mode_ignores_an_unstaged_comment() {
    let fixture = Fixture::init();
    fixture
        .write("src/lib.rs", "fn f() {}\n")
        .edit_unstaged("src/lib.rs", "fn f() {} // why\n");
    let outcome = fixture.xtask(&["check-rules", "--staged"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
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

#[test]
fn shell_script_without_an_extension_is_checked() {
    assert_rejected("build-aux/run", "#!/bin/sh\n# why\n");
}

#[test]
fn python_comment_is_rejected() {
    assert_rejected("tests/mock.py", "value = 1  # why\n");
}

#[test]
fn containerfile_comment_is_rejected() {
    assert_rejected("build-aux/Containerfile", "# why\nFROM fedora\n");
}

#[test]
fn dockerfile_comment_is_rejected() {
    assert_rejected("Dockerfile", "# why\nFROM fedora\n");
}

#[test]
fn ini_comment_is_rejected() {
    assert_rejected("data/settings.ini", "# why\n[section]\n");
}

#[test]
fn gtk_builder_comment_is_rejected() {
    assert_rejected(
        "crates/ui/resources/window.ui",
        "<interface><!-- why --></interface>\n",
    );
}

#[test]
fn svg_comment_is_rejected() {
    assert_rejected("data/icons/app.svg", "<svg><!-- why --></svg>\n");
}

#[test]
fn policy_comment_is_rejected() {
    assert_rejected(
        "data/app.policy",
        "<policyconfig><!-- why --></policyconfig>\n",
    );
}

#[test]
fn markdown_html_comment_is_rejected() {
    assert_rejected("docs/notes.md", "# Notes\n\nText <!-- why -->\n");
}

#[test]
fn markdown_comment_marker_inside_code_is_accepted() {
    assert_accepted(
        "docs/notes.md",
        "Write `<!--` in XML.\n\n```xml\n<!-- example -->\n```\n",
    );
}

#[test]
fn commonjs_comment_is_rejected() {
    assert_rejected(
        "extension/eslint.config.cjs",
        "module.exports = {}; // why\n",
    );
}

#[test]
fn typescript_comment_is_rejected() {
    assert_rejected("extension/lib/types.ts", "/* why */\nexport type A = 1;\n");
}

#[test]
fn scss_line_comment_is_rejected() {
    assert_rejected("crates/ui/resources/style.scss", "// why\n.row {}\n");
}

#[test]
fn css_unquoted_url_is_accepted() {
    assert_accepted(
        "crates/ui/resources/style.css",
        ".row { background: url(http://example.org/a.png); }\n",
    );
}

#[test]
fn javascript_template_literal_holding_slashes_is_accepted() {
    assert_accepted(
        "extension/lib/client.js",
        "export const url = `http://example.org`;\n",
    );
}

#[test]
fn unknown_text_file_is_reported() {
    let fixture = support::Fixture::init();
    fixture.write("data/thing.weird", "text\n");
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(!outcome.succeeded());
    assert!(
        outcome.stderr.contains("data/thing.weird: unknown syntax"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn plain_text_json_and_hex_dumps_are_accepted() {
    assert_accepted("notes/todo.txt", "# not a comment\n");
    assert_accepted("docs/research/captures/reply.hex", "27 00 00\n");
    assert_accepted("data/value.json", "{\"color\": \"#fff\"}\n");
}

#[test]
fn csv_data_is_plain_text() {
    assert_accepted(
        "crates/core/data/curve.csv",
        "freq_hz,spl_db\n20.00,-4.25\n",
    );
}

#[test]
fn hash_inside_a_word_is_accepted() {
    assert_accepted("build-aux/run.sh", "#!/bin/sh\necho C# $#\n");
}

#[test]
fn comment_after_an_escaped_quote_is_rejected() {
    assert_rejected("config/settings.toml", "value = \"a\\\"\" # why\n");
}

#[test]
fn comment_after_an_escaped_backslash_is_rejected() {
    assert_rejected("config/settings.toml", "value = \"a\\\\\" # why\n");
}

#[test]
fn comment_after_a_literal_string_ending_in_a_backslash_is_rejected() {
    assert_rejected("config/settings.toml", "path = 'C:\\' # why\n");
}

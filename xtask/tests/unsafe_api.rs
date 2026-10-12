mod support;

use support::{assert_accepted, assert_rejected};

const PORT: &str = "crates/hid/src/port.rs";

#[test]
fn pub_unsafe_fn_is_rejected() {
    assert_rejected(PORT, "pub unsafe fn read() {}\n");
}

#[test]
fn restricted_pub_unsafe_fn_is_rejected() {
    assert_rejected(PORT, "pub(crate) unsafe fn read() {}\n");
}

#[test]
fn qualified_pub_unsafe_extern_fn_is_rejected() {
    assert_rejected(PORT, "pub const unsafe extern \"C\" fn read() {}\n");
}

#[test]
fn private_unsafe_fn_is_accepted() {
    assert_accepted(PORT, "unsafe fn read() {}\npub fn safe() {}\n");
}

#[test]
fn unsafe_block_inside_a_safe_pub_fn_is_accepted() {
    assert_accepted(
        PORT,
        "pub fn read() -> u8 {\n    unsafe { core::ptr::read(&1) }\n}\n",
    );
}

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

#[test]
fn pub_fn_in_an_extern_block_is_rejected() {
    assert_rejected(
        PORT,
        "unsafe extern \"C\" {\n    pub fn ioctl(fd: i32) -> i32;\n}\n",
    );
}

#[test]
fn pub_static_in_an_extern_block_is_rejected() {
    assert_rejected(PORT, "extern \"C\" {\n    pub static errno: i32;\n}\n");
}

#[test]
fn pub_safe_fn_in_an_extern_block_is_accepted() {
    assert_accepted(
        PORT,
        "unsafe extern \"C\" {\n    pub safe fn abs(value: i32) -> i32;\n}\n",
    );
}

#[test]
fn private_fn_in_an_extern_block_is_accepted() {
    assert_accepted(
        PORT,
        "unsafe extern \"C\" {\n    fn ioctl(fd: i32) -> i32;\n}\npub fn safe() {}\n",
    );
}

#[test]
fn pub_items_after_an_extern_block_are_accepted() {
    assert_accepted(
        PORT,
        "unsafe extern \"C\" {\n    fn ioctl(fd: i32) -> i32;\n}\npub fn safe() {}\npub static VALUE: i32 = 1;\n",
    );
}

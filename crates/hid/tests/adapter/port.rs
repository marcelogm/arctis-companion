use std::fs;

use arctis_core::protocol::{Command, Query, encode};
use arctis_hid::port::{Port, PortGone, Received};
use rustix::net::Shutdown;

use crate::sockets::{pair, receive, send};

#[test]
fn a_write_sends_the_whole_report() {
    let (ours, theirs) = pair();
    let port = Port::from_fd(ours);
    let frame = encode(&Command::from(Query::Status));
    port.write(&frame).expect("written");
    assert_eq!(receive(&theirs), Some(frame.as_bytes().to_vec()));
}

#[test]
fn a_read_returns_one_frame() {
    let (ours, theirs) = pair();
    let port = Port::from_fd(ours);
    send(&theirs, &[0xB0, 0x02, 0x02]);
    let Ok(Received::Report(bytes)) = port.read() else {
        panic!("a frame");
    };
    assert_eq!(bytes, [0xB0, 0x02, 0x02]);
}

#[test]
fn a_read_with_nothing_pending_is_empty() {
    let (ours, _theirs) = pair();
    assert!(matches!(Port::from_fd(ours).read(), Ok(Received::Empty)));
}

#[test]
fn a_read_after_the_peer_left_is_gone() {
    let (ours, theirs) = pair();
    drop(theirs);
    assert_eq!(Port::from_fd(ours).read().err(), Some(PortGone));
}

#[test]
fn a_write_the_peer_refuses_is_gone() {
    let (ours, theirs) = pair();
    rustix::net::shutdown(&theirs, Shutdown::Read).expect("shutdown");
    let frame = encode(&Command::from(Query::Status));
    assert_eq!(Port::from_fd(ours).write(&frame), Err(PortGone));
}

#[test]
fn a_regular_file_opens_as_a_port() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let node = dir.path().join("hidraw6");
    fs::write(&node, b"").expect("a stand-in node");
    assert!(Port::open(&node).is_ok());
}

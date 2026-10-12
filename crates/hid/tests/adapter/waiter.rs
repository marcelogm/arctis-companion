use std::os::fd::AsFd;

use arctis_hid::waiter::{PollWaiter, Waiter};

use crate::sockets::{pair, send};

#[test]
fn a_readable_fd_is_reported_ready() {
    let (ours, theirs) = pair();
    let (quiet, _other) = pair();
    send(&theirs, &[1]);
    let mut waiter = PollWaiter::new();
    let deadline = waiter.now().after(1_000);
    assert_eq!(
        waiter.wait(&[quiet.as_fd(), ours.as_fd()], Some(deadline)),
        [false, true]
    );
}

#[test]
fn a_hung_up_fd_is_reported_ready() {
    let (ours, theirs) = pair();
    drop(theirs);
    let mut waiter = PollWaiter::new();
    assert_eq!(waiter.wait(&[ours.as_fd()], None), [true]);
}

#[test]
fn the_wait_ends_at_the_deadline() {
    let (ours, _theirs) = pair();
    let mut waiter = PollWaiter::new();
    let deadline = waiter.now().after(30);
    assert_eq!(waiter.wait(&[ours.as_fd()], Some(deadline)), [false]);
    assert!(waiter.now() >= deadline);
}

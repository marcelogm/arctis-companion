use std::os::fd::AsFd;

use arctis_core::messages::device::{Change, SyncPolicy};
use arctis_core::testkit::FakeHeadset;
use arctis_hid::wake::{self, Mail};

use crate::sockets::poll_now;

fn ringing(mailbox: &impl AsFd) -> bool {
    poll_now(&[mailbox.as_fd()])[0]
}

fn change() -> Change {
    Change::Desired {
        settings: FakeHeadset::nova_5().settings().clone(),
        policy: SyncPolicy::Enforce,
    }
}

#[test]
fn a_sent_change_rings_the_mailbox() {
    let (changes, mailbox) = wake::mailbox().expect("a mailbox");
    assert!(!ringing(&mailbox));
    assert!(changes.send(change()));
    assert!(ringing(&mailbox));
}

#[test]
fn taking_the_mail_returns_every_change_and_drains_the_wakes() {
    let (changes, mailbox) = wake::mailbox().expect("a mailbox");
    assert!(changes.send(change()));
    assert!(changes.send(change()));
    assert_eq!(
        mailbox.take(),
        Mail {
            changes: vec![change(), change()],
            quit: false,
        }
    );
    assert!(!ringing(&mailbox));
}

#[test]
fn a_quit_message_ends_the_mail() {
    let (changes, mailbox) = wake::mailbox().expect("a mailbox");
    changes.quit();
    assert!(changes.send(change()));
    assert_eq!(
        mailbox.take(),
        Mail {
            changes: Vec::new(),
            quit: true,
        }
    );
}

#[test]
fn dropping_every_sender_means_quit() {
    let (changes, mailbox) = wake::mailbox().expect("a mailbox");
    drop(changes);
    assert!(mailbox.take().quit);
}

#[test]
fn a_change_sent_after_the_worker_left_reports_failure() {
    let (changes, mailbox) = wake::mailbox().expect("a mailbox");
    drop(mailbox);
    assert!(!changes.send(change()));
}

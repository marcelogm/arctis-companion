use arctis_core::messages::device::Phase;
use arctis_core::session::timing::{RESCAN_ABSENT, RESCAN_REBINDING};
use arctis_core::testkit::{FakeHeadset, Rig};

use crate::rig::SocketRig;

const UDEV_ACL_DELAY: u64 = 60;

#[test]
fn a_node_denied_until_udev_applies_its_acl_never_shows_permission_denied() {
    let mut rig = SocketRig::start(FakeHeadset::nova_5());
    rig.plug_with_permission_after(UDEV_ACL_DELAY);
    rig.run_until(Phase::Settling, RESCAN_ABSENT);
    assert_eq!(rig.published_phases(), [Phase::Settling]);
}

#[test]
fn a_node_that_stays_denied_shows_permission_denied_after_one_quick_retry() {
    let mut rig = SocketRig::start(FakeHeadset::nova_5());
    rig.deny();
    let waited = rig.run_until(Phase::PermissionDenied, RESCAN_ABSENT);
    assert!(waited >= RESCAN_REBINDING);
    assert_eq!(rig.published_phases(), [Phase::PermissionDenied]);
}

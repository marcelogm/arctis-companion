use arctis_core::protocol::model::{Model, ModelLookup};
use arctis_hid::SysfsRoot;
use arctis_hid::discovery;

use crate::hardware_enabled;

#[test]
#[ignore = "needs the real dongle: ARCTIS_HW=1"]
fn the_real_dongle_is_found_in_sysfs() {
    if !hardware_enabled() {
        return;
    }
    let nodes = discovery::scan(&SysfsRoot::system());
    let dongle = discovery::first_dongle(&nodes).expect("a supported dongle is plugged in");
    eprintln!("found {dongle:?}");
    assert!(matches!(
        Model::lookup(dongle.pid),
        ModelLookup::Supported(_)
    ));
}

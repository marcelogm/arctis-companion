use arctis_core::testkit::scenarios;

use crate::rig::SocketRig;

#[test]
fn the_worker_passes_the_scenario_catalogue_over_sockets() {
    scenarios::run_all::<SocketRig>();
}

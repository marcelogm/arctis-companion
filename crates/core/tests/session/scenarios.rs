use arctis_core::testkit::{Simulation, scenarios};

#[test]
fn the_scenario_catalogue_holds_on_the_simulation() {
    scenarios::run_all::<Simulation>();
}

#[test]
fn the_scenario_catalogue_names_each_scenario_once() {
    let mut names: Vec<_> = scenarios::catalogue::<Simulation>()
        .iter()
        .map(|scenario| scenario.name)
        .collect();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count);
}

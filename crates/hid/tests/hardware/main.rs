mod discovery;
mod soak;
mod soak_log;

const HARDWARE: &str = "ARCTIS_HW";

fn hardware_enabled() -> bool {
    let enabled = std::env::var(HARDWARE).is_ok_and(|value| value == "1");
    if !enabled {
        eprintln!("skipped: set {HARDWARE}=1 with the dongle plugged in");
    }
    enabled
}

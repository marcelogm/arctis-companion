use arctis_core::session::Cadence;
use arctis_core::session::timing::{RESCAN_ABSENT, RESCAN_REBINDING};
use arctis_core::time::Millis;
use arctis_hid::rescan::Rescan;

#[test]
fn the_first_scan_is_due_at_start() {
    let mut rescan = Rescan::starting_at(Millis(0));
    assert_eq!(rescan.take_due(Millis(0)), Some(Cadence::Absent));
    assert_eq!(rescan.next(), Some(Millis(RESCAN_ABSENT)));
}

#[test]
fn each_scanning_cadence_repeats_at_its_interval() {
    for (cadence, interval) in [
        (Cadence::Absent, RESCAN_ABSENT),
        (Cadence::Rebinding, RESCAN_REBINDING),
    ] {
        let mut rescan = Rescan::starting_at(Millis(0));
        rescan.set(cadence, Millis(10));
        assert_eq!(rescan.take_due(Millis(9 + interval)), None);
        assert_eq!(rescan.take_due(Millis(10 + interval)), Some(cadence));
        assert_eq!(rescan.next(), Some(Millis(10 + 2 * interval)));
    }
}

#[test]
fn cadence_off_never_scans() {
    let mut rescan = Rescan::starting_at(Millis(0));
    rescan.set(Cadence::Off, Millis(0));
    assert_eq!(rescan.next(), None);
    assert_eq!(rescan.take_due(Millis(u64::MAX)), None);
}

#[test]
fn a_quick_retry_comes_once_then_the_cadence_resumes() {
    let mut rescan = Rescan::starting_at(Millis(0));
    assert_eq!(rescan.take_due(Millis(0)), Some(Cadence::Absent));
    rescan.retry_soon(Millis(0));
    assert_eq!(rescan.next(), Some(Millis(RESCAN_REBINDING)));
    assert_eq!(
        rescan.take_due(Millis(RESCAN_REBINDING)),
        Some(Cadence::Absent)
    );
    assert_eq!(
        rescan.next(),
        Some(Millis(RESCAN_REBINDING + RESCAN_ABSENT))
    );
}

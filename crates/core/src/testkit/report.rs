use crate::protocol::Frame;
use crate::protocol::frame::{PAYLOAD_LEN, REPORT_LEN};

#[must_use]
pub fn frame_from_report(report: &[u8]) -> Option<Frame> {
    if report.len() != REPORT_LEN || report[0] != 0 {
        return None;
    }
    let args: &[u8; PAYLOAD_LEN - 1] = report[2..].try_into().ok()?;
    Some(Frame::new(report[1], args))
}

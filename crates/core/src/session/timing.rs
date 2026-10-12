pub const COMMAND_SPACING: u64 = 50;
pub const ENUMERATION_SETTLE: u64 = 3_000;
pub const REBIND_SETTLE: u64 = 100;
pub const REBIND_NOTICE: u64 = 5_000;
pub const REBIND_GIVE_UP: u64 = 30_000;
pub const REPLY_TIMEOUT: u64 = 500;
pub const VERIFY_BACKOFF: [u64; 3] = [50, 150, 400];
pub const ECHO_WINDOW: u64 = 1_000;
pub const WIRELESS_MODE_GRACE: u64 = 5_000;
pub const HEADSET_OFF_POLL: u64 = 10_000;
pub const STATUS_SAFETY_POLL: u64 = 300_000;
pub const STATUS_SAFETY_POLL_FLOOR: u64 = 60_000;
pub const RESCAN_ABSENT: u64 = 2_000;
pub const RESCAN_REBINDING: u64 = 250;

const _: () = assert!(STATUS_SAFETY_POLL >= STATUS_SAFETY_POLL_FLOOR);

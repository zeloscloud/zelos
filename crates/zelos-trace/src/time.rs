use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current time as nanoseconds from epoch
/// Panics on clock going backwards
pub fn now_time_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| panic!("Time went backwards"))
        .as_nanos() as i64
}

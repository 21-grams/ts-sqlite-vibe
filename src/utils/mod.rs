pub mod error;
pub mod csv;
#[cfg(test)]
pub mod test_utils;

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}
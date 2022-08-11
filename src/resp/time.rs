use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_millis_since_epoch() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

use std::time::{SystemTime, UNIX_EPOCH};

/// Create an (almost) unique string without use of external crates
///
/// Based on https://users.rust-lang.org/t/random-number-without-using-the-external-crate/17260/9
pub fn unique_string() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize;

    let ptr1 = Box::into_raw(Box::new(0)) as usize;
    let ptr2 = Box::into_raw(Box::new(0)) as usize;

    format!("{:x}", nanos + ptr1 + ptr2)
}

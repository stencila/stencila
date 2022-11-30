/// Should test be skipped on CI?
pub fn skip_ci(reason: &str) -> bool {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping test on CI: {}", reason);
        true
    } else {
        false
    }
}

/// Should test be skipped on the current operating system?
///
/// See https://doc.rust-lang.org/std/env/consts/constant.OS.html for
/// possible values.
pub fn skip_os(os: &str, reason: &str) -> bool {
    if std::env::consts::OS == os {
        eprintln!("Skipping test on OS `{}`: {}", os, reason);
        true
    } else {
        false
    }
}

/// Should test be skipped on CI for an operating system?
pub fn skip_ci_os(os: &str, reason: &str) -> bool {
    if std::env::var("CI").is_ok() && std::env::consts::OS == os {
        eprintln!("Skipping test on CI for OS `{}`: {}", os, reason);
        true
    } else {
        false
    }
}

/// Should slow tests be skipped?
///
/// Use at the start of slow tests to return early except on CI or when
/// the env var `RUN_SLOW_TESTS` is set.
///
/// Inspired by https://github.com/rust-analyzer/rust-analyzer/pull/2491
pub fn skip_slow() -> bool {
    if std::env::var("CI").is_err() && std::env::var("RUN_SLOW_TESTS").is_err() {
        eprintln!("Skipping slow test");
        true
    } else {
        false
    }
}

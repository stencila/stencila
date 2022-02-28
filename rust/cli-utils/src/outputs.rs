// Is standard output stream a TTY?
pub fn stdout_isatty() -> bool {
    atty::is(atty::Stream::Stdout)
}

// Is standard error stream a TTY?
pub fn stderr_isatty() -> bool {
    atty::is(atty::Stream::Stderr)
}

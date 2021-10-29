/// Ensure that a string is a valid JSON pointer
///
/// Replaces dots (`.`) with slashes (`/`) and ensures a
/// leading slash.
pub fn pointer(pointer: &str) -> String {
    let pointer = pointer.replace(".", "/");
    if pointer.starts_with('/') {
        pointer
    } else {
        ["/", &pointer].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_pointer() {
        assert_eq!(pointer("a"), "/a");
        assert_eq!(pointer("a/b"), "/a/b");
        assert_eq!(pointer("/a.b"), "/a/b");
        assert_eq!(pointer("a.b.c"), "/a/b/c");
    }
}

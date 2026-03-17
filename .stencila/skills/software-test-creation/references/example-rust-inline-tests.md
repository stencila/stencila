# Example: Rust crate with inline test module

Slice: "Phase 1 / Slice 1" — Define `AuthToken` and `AuthError` types
Acceptance criteria: `AuthToken` struct has `sub`, `exp`, `iat`, `roles` fields; `AuthError` enum has `MalformedToken` variant
Package: `rust/auth`

**Discovery**: Exploration reveals the crate uses inline `#[cfg(test)] mod tests` blocks with `#[test]` functions and `assert_eq!`. The `Cargo.toml` has `tokio` as a dev-dependency. The `Makefile` has a `test` target that runs `cargo test -p auth`.

Test written in `rust/auth/src/types.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_token_has_required_fields() {
        let token = AuthToken {
            sub: "user-123".to_string(),
            exp: 1700000000,
            iat: 1699999000,
            roles: vec!["admin".to_string()],
        };
        assert_eq!(token.sub, "user-123");
        assert_eq!(token.exp, 1700000000);
        assert_eq!(token.iat, 1699999000);
        assert_eq!(token.roles, vec!["admin".to_string()]);
    }

    #[test]
    fn auth_error_has_malformed_token_variant() {
        let err = AuthError::MalformedToken;
        assert!(matches!(err, AuthError::MalformedToken));
    }
}
```

Context stored:

- `slice.test_files` = `rust/auth/src/types.rs`
- `slice.test_command` = `cargo test -p auth`

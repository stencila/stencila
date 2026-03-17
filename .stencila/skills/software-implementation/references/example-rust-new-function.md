# Example: Rust — new function in existing module

Slice: "Phase 1 / Slice 2" — Add token validation
Acceptance criteria: `validate_token` takes a `&str` and returns `Result<Claims, AuthError>`; rejects expired tokens with `AuthError::Expired`; extracts roles into `Claims.roles`
Package: `rust/auth`

**Test failure output**:
```
error[E0433]: failed to resolve: could not find `validate_token` in `auth`
 --> src/auth/tests.rs:5:21
  |
5 |     use crate::auth::validate_token;
  |                       ^^^^^^^^^^^^^^ not found in `auth`
```

**Discovery**: Read `src/auth/mod.rs` — has existing functions `create_token` and `decode_token` following the pattern of taking `&str` input and returning `Result<T, AuthError>`. The `AuthError` enum and `Claims` struct are already defined in the same module. The codebase uses `thiserror` for error derivation and inline `#[cfg(test)] mod tests` blocks.

**Action**:
1. Read test file — `validate_token` takes a `&str`, returns `Result<Claims, AuthError>`
2. Study `create_token` and `decode_token` for conventions (error type, Claims struct, use of `jsonwebtoken` crate)
3. Add `validate_token` function to `src/auth/mod.rs` with the expected signature, following the existing function style
4. Run `cargo check -p auth` — clean

**Summary**: Modified `src/auth/mod.rs`, added `validate_token` function. Follows existing pattern of `decode_token` but adds validation logic. Uses the same `AuthError` and `Claims` types already defined in the module.

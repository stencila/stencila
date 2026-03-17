# Example: Revision after partial implementation

This example continues from the Rust token validation scenario in `references/example-rust-new-function.md`. The initial implementation made 2 of 4 tests pass, and revision feedback identifies the remaining failures.

Slice: "Phase 1 / Slice 2" — Add token validation (revision)
Package: `rust/auth`

**Revision feedback**:
```
2 of 4 tests now pass. Remaining failures:
- test_validate_rejects_expired_token: Expected AuthError::Expired, got AuthError::Invalid
- test_validate_extracts_roles: assertion `claims.roles == vec!["admin", "user"]` failed
```

**Discovery**: Re-read the current `validate_token` implementation in `src/auth/mod.rs`. The function decodes the token and validates the signature but:
- Does not check the `exp` field against the current time — all invalid tokens return `AuthError::Invalid` regardless of the failure reason
- Does not parse the `roles` claim from the token payload into `Claims.roles` — the field is left as an empty vec

**Action**:
1. Read the test file to understand exactly what is expected:
   - `test_validate_rejects_expired_token` creates a token with `exp` in the past and expects `AuthError::Expired`
   - `test_validate_extracts_roles` creates a token with `roles: ["admin", "user"]` and asserts `claims.roles` matches
2. Add expiration check to `validate_token`: compare `exp` against current time, return `AuthError::Expired` (not `AuthError::Invalid`) when expired
3. Add roles extraction: deserialize the `roles` claim from the JWT payload into `Claims.roles`
4. Run `cargo check -p auth` — clean

**Summary**: Modified `src/auth/mod.rs`. Added expiration check returning `AuthError::Expired` for tokens past their `exp` time. Added roles extraction from JWT payload into `Claims.roles` field. Both changes follow the existing error handling pattern in the module.

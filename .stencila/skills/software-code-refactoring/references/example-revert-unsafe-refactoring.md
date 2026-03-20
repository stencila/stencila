# Example: Revert an unsafe refactoring

This example shows how to handle a refactoring that breaks tests.

Target files: `src/cache/lru.rs`
Test command: `cargo test -p cache`
Refactoring focus: simplify complexity

**Discovery**: Read `src/cache/lru.rs` — an `evict` method has a complex conditional with 4 branches handling different eviction strategies based on entry age and access frequency.

**Analysis**:
- Complexity: the 4-branch conditional in `evict` could potentially be simplified by combining the age and frequency checks
- The logic is subtle — different orderings of the checks produce different eviction behavior

**Action**:
1. Simplify the `evict` method by combining the age check and frequency check into a single scoring expression and sorting by score
2. Run `cargo check -p cache` — clean
3. Run `cargo test -p cache` — **2 failures**:
   - `test_evict_prefers_old_over_frequent`: expected entry A evicted, got entry B
   - `test_evict_tiebreak_by_insertion_order`: expected entry C evicted, got entry D
4. The simplified scoring changes the eviction priority in edge cases — the original branching order was intentional
5. **Revert**: restore the original `evict` method
6. Run `cargo test -p cache` — all 23 tests pass (confirming revert is clean)
7. Apply a safer refactoring instead: extract each branch body into a descriptively named helper method (`evict_by_age`, `evict_by_frequency`, `evict_stale`, `evict_lru`), keeping the original branch structure
8. Run `cargo test -p cache` — all 23 tests pass

**Summary**: Modified `src/cache/lru.rs`. Attempted to simplify `evict` by combining branch conditions into a scoring expression — reverted because it changed eviction priority (2 test failures). Instead, extracted 4 helper methods from the branch bodies, preserving the original control flow while improving readability. Category: complexity (readability). Reverted refactoring: condition simplification (unsafe — changed behavior). No public API changes. All tests pass.

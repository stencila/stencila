# Fallback test conventions

Use this reference only when no existing test conventions can be discovered in the target package, its siblings, or the project root. If the codebase already has tests, follow those conventions instead.

When using a fallback framework, install or declare the minimum required test framework package(s) if they are not already available. The generated red-phase tests should fail because the implementation is missing or incorrect, not because the test runner itself is absent.

| Language      | Framework          | Test File Pattern                        | Test Command                               | Assertion Example                  |
| ------------- | ------------------ | ---------------------------------------- | ------------------------------------------ | ---------------------------------- |
| C/C++         | GoogleTest         | `tests/*_test.cpp`                       | `ctest --test-dir build`                   | `EXPECT_EQ(actual, expected)`      |
| C#            | xUnit              | `*.Tests/*Tests.cs`                      | `dotnet test`                              | `Assert.Equal(expected, actual)`   |
| Go            | built-in `testing` | `*_test.go`                              | `go test ./...`                            | `if got != want { t.Errorf(...) }` |
| Java          | JUnit 5            | `src/test/java/**/*Test.java`            | `mvn test`                                 | `assertEquals(expected, actual)`   |
| Kotlin        | JUnit 5            | `src/test/kotlin/**/*Test.kt`            | `gradle test`                              | `assertEquals(expected, actual)`   |
| Python        | pytest             | `tests/test_*.py`                        | `pytest tests/`                            | `assert result == expected`        |
| R             | testthat           | `tests/testthat/test-*.R`                | `Rscript -e "testthat::test_dir('tests')"` | `expect_equal(actual, expected)`   |
| Ruby          | RSpec              | `spec/**/*_spec.rb`                      | `bundle exec rspec`                        | `expect(result).to eq(expected)`   |
| Rust          | built-in `#[test]` | `#[cfg(test)] mod tests` or `tests/*.rs` | `cargo test -p <crate>`                    | `assert_eq!(actual, expected)`     |
| Scala         | ScalaTest          | `src/test/scala/**/*Spec.scala`          | `sbt test`                                 | `result shouldBe expected`         |
| Shell         | Bats               | `tests/*.bats`                           | `bats tests/`                              | `[ "$output" = "expected" ]`       |
| Swift         | XCTest             | `Tests/**/*Tests.swift`                  | `swift test`                               | `XCTAssertEqual(actual, expected)` |
| TypeScript/JS | Vitest             | `*.test.ts` or `__tests__/*.test.ts`     | `npx vitest run`                           | `expect(result).toBe(expected)`    |

# Framework Detection Reference

When `slice.test_command` is not available in workflow context, use this reference to detect the test framework and construct the correct command.

## Detection by Build File

| Build File          | Language      | Default Test Command               | Scoping Pattern                                    |
| ------------------- | ------------- | ---------------------------------- | -------------------------------------------------- |
| `Cargo.toml`        | Rust          | `cargo test`                       | `cargo test -p <crate>`                            |
| `go.mod`            | Go            | `go test ./...`                    | `go test ./<package>/...`                           |
| `package.json`      | JS/TS         | Check `scripts.test` in the file   | Append `-- --testPathPattern=<pattern>`             |
| `pyproject.toml`    | Python        | `pytest` or check `[tool.pytest]`  | `pytest <test-file-or-dir>`                        |
| `setup.py`/`tox.ini`| Python        | `pytest` or `python -m pytest`     | `pytest <test-file-or-dir>`                        |
| `pom.xml`           | Java          | `mvn test`                         | `mvn test -pl <module> -Dtest=<TestClass>`         |
| `build.gradle*`     | Java/Kotlin   | `gradle test`                      | `gradle :module:test --tests <pattern>`            |
| `Gemfile`           | Ruby          | `bundle exec rspec`                | `bundle exec rspec <spec-file>`                    |
| `mix.exs`           | Elixir        | `mix test`                         | `mix test <test-file>`                             |
| `Package.swift`     | Swift         | `swift test`                       | `swift test --filter <TestCase>`                   |
| `*.csproj`          | C#            | `dotnet test`                      | `dotnet test --filter <FullyQualifiedName>`        |
| `DESCRIPTION`       | R             | `Rscript -e "testthat::test_dir('tests')"` | `Rscript -e "testthat::test_file('<file>')"`|
| `CMakeLists.txt`    | C/C++         | `ctest --test-dir build`           | `ctest --test-dir build -R <pattern>`              |

## Detection by Test File Pattern

If the test command is missing but `slice.test_files` is available, infer the framework from the file extensions and paths:

| File Pattern                   | Likely Framework     | Command Pattern                      |
| ------------------------------ | -------------------- | ------------------------------------ |
| `*.rs` with `#[test]`         | Rust built-in        | `cargo test -p <crate>`             |
| `*_test.go`                   | Go built-in          | `go test ./<dir>/...`               |
| `test_*.py` / `*_test.py`    | pytest               | `pytest <file>`                      |
| `*.test.ts` / `*.spec.ts`    | vitest or jest       | Check package.json for which runner  |
| `*.test.js` / `*.spec.js`    | vitest or jest       | Check package.json for which runner  |
| `*_spec.rb`                   | RSpec                | `bundle exec rspec <file>`           |
| `*Test.java`                  | JUnit                | `mvn test -Dtest=<Class>`           |
| `*_test.exs`                  | ExUnit               | `mix test <file>`                    |
| `*.bats`                      | Bats                 | `bats <file>`                        |

## Scoping Strategy

Always prefer scoped commands over full-suite runs:

1. If `slice.test_files` lists specific files, run only those files
2. If `slice.packages` names specific packages or modules, restrict the test command to those
3. If neither is available, look at `slice.scope` for hints about which package or module is relevant
4. Only fall back to a full test suite run as a last resort, and note this in the output

## Makefile and CI Detection

Check these locations for canonical test commands:

- `Makefile` — look for `test:` or `check:` targets, especially scoped ones like `make -C <dir> test`
- `.github/workflows/*.yml` — look for `run:` steps that execute tests
- `.gitlab-ci.yml` — look for `script:` entries with test commands
- `package.json` `scripts.test` — the project's configured test runner
- `tox.ini` / `nox` — Python test orchestrators

# Example: R package with testthat

Slice: "Phase 1 / Slice 3" — Add URL normalization helper
Acceptance criteria: `normalize_url` trims whitespace, lowercases the scheme and host, preserves the path, and errors on malformed input
Package: `R/`

**Discovery**: No existing tests are found in the package, but the project structure and R sources indicate an R package layout. The fallback conventions are applied for R with testthat. Because the package does not yet declare testthat, the agent adds the minimal test framework setup needed to run the tests, such as adding `testthat` to `Suggests` and enabling `Config/testthat/edition` in `DESCRIPTION` if needed.

Test written in `tests/testthat/test-normalize-url.R`:

```r
test_that("normalize_url trims whitespace and normalizes scheme and host", {
  expect_equal(
    normalize_url("  HTTPS://Example.COM/path  "),
    "https://example.com/path"
  )
})

test_that("normalize_url errors on malformed input", {
  expect_error(normalize_url("not a url"))
})
```

Context stored:

- `slice.test_files` = `tests/testthat/test-normalize-url.R`
- `slice.test_command` = `Rscript -e "testthat::test_dir('tests')"`

Summary notes: "No existing R test conventions were found. Applied fallback conventions for R with testthat and added the minimal test framework dependency/setup required so the tests fail for missing implementation rather than missing test infrastructure."

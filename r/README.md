# Stencila Schema for R

This package provides R bindings for [Stencila Schema](https://stencila.github.io/schema/).

## Install

This package isn't on CRAN yet, but you can install it from this repository using the [`devtools`](https://github.com/hadley/devtools) package,

```r
devtools::install_github("stencila/schema", subdir = "r", upgrade = "ask")
```

## Develop

Most development tasks can be run from R, using `make` shortcuts, or RStudio keyboard shortcuts.

| Task                             | `make`          | R/RStudio                             |
| -------------------------------- | --------------- | ------------------------------------- |
| Install development dependencies | `make setup`    |
| Regenerate `R/types.R`           | `make regen`    |
| Run linting                      | `make lint`     | `lintr::lint_package()`               |
| Run tests                        | `make test`     | `devtools::test()` or `Ctrl+Shift+T`  |
| Re-run tests on changes          | `make autotest` | `testthat::auto_test_package()`       |
| Run tests with coverage          | `make cover`    | `covr::package_coverage()`            |
| Build documentation              | `make docs`     | `devtools::document()`                |
| Check the package                | `make check`    | `Ctrl+Shift+E`                        |
| Build the package                | `make build`    | `devtools::build()` or `Ctrl+Shift+B` |
| Clean                            | `make clean`    |

Unit tests live in the `tests` folder and are written using the `testthat` package. To run test files individually, in R use the `test_file` function:

```r
testthat::test_file(system.file("tests/testthat/test-types.R", package = "stencila"))
```

The tests are run on [Travis](https://travis-ci.org/stencila/schema) and code coverage tracked at [Codecov](https://codecov.io/gh/stencila/schema).

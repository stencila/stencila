# R
You can write the contents of Stencila cells in R as well as make your existing R functions available in Stencila through [function libraries][libraries-contribute]. An execution context for R, `RContext`. Currrently, in order to have `RContext` available, you need to have an R sesssion running either locally or point to a session running in a remote location.

## Data interchange

## Cells

## Functions
You can make almost any R functions for data manipulation available from within Stencila by using our API (which is a simple wrapper). You can either contribute new functions to the Stencila Core Library, existing domain-specific libraries or create a new domain-specific library and add your functions there.


### Implement

In the revelant `.fun.xml` file (an existing one, or one which you created above) add a `<implem language="r">` element under `<implems>`. This registers your R implementation with the Stencila execution engine.

Create the R function implementation in a `.R` file in the `R` folder e.g. `R/sum.R` for the `sum` function.

### Test

To test your function implementation, create a new test file in the `tests/testthat` folder e.g. `tests/testthat/test_sum.R` for the `sum` function.

Install some useful R packages for package development and testing, if you don't already have them,

```r
install.packages(c('devtools', 'roxygen2', 'lintr', 'testthat', 'covr'))
```

Check for lint,

```bash
lintr::lint_package()
```

Then run all the tests,

```bash
devtools::test()
```

Alternatively, you can run the tests and calculate test coverage using,

```bash
covr::package_coverage()
```

### Register


[libraries-contribute]: computation/functions.md#domain-specific-libraries

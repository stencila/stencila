# R

Cells and functions can be written using Javascript. An execution context for Javascript, `JsContext` is implemented in the [`stencila/stencila`](https://github.com/stencila/stencila) repository so it's already availble to all Stencila documents without the need for external hosts.

## Data interchange



## Cells



## Functions

TODO - revise!

### Create

See the [`meta`](../meta) folder for which functions are the highet priority for implementation. Each function needs an XML file in the `../xml` folder. If necessary, create a new function using `../xml/.fun.xml` as a template.

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
``

Then run all the tests,

```bash
devtools::test()
```

Alternatively, you can run the tests and calculate test coverage using,

```bash
covr::package_coverage()
```

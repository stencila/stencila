# R
You can write the contents of Stencila cells in R as well as make your existing R functions available in Stencila through [function libraries][libraries-contribute]. An execution context for R, `RContext`. Currrently, in order to have `RContext` available, you need to have an R sesssion running either locally or point to a session running in a remote location.

## Data interchange

## Cells

## Functions
You can make almost any R functions for data manipulation available from within Stencila by using our API (which is a simple wrapper). You can either contribute new functions to the Stencila Core Library, existing domain-specific libraries or create a new domain-specific library and add your functions there.


### Implement



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

### Register :sparkles:
Once you finished implementing and testing your R function, you need to register it to make it available from within Stencila. You can do it either through RStudio
or via Stencila Sheets.

See the demo below how to register functions from RStudio.




In order to do
that select `Register function` from the menu and point to the main directory (for example, `libgenomics`) where the `.py` file with the function is located. Stencila will automatically
 create the documentation from the docstring. You can then use the function within Stencila.

 If you want to make the function available for someone else using Stencila on a different machine, select `Create function package`, then point
 to the man directory with function file. Once the function package is created, select where you want to save it. You can then share the package (which
 essentially is a `zip` file). If the main directory with the function exists as a GitHub or BitBucket repository (see [these guidelines](https://github.com/stencila/libtemplate)),
 then you can simply point users to the repository.

 To register the function from the package, use the `Register function` option from the menu, as described above. If you are registering from a GitHub or BitBucket repository,
 just copy and paste the link to it.

[libraries-contribute]: computation/functions.md#domain-specific-libraries

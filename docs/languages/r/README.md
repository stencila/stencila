# R
You can write the contents of Stencila cells in R as well as make your existing R functions available in Stencila through [function libraries][libraries-contribute]. An execution context for R, `RContext`. Currrently, in order to have `RContext` available, you need to have an R sesssion running either locally or point to a session running in a remote location.

## Data interchange

## Cells

With Stencila you have full control over the sequence in which your code cells are executed. You can run the code in asynchronous order.
You can refer to specific outputs from the given cell in any part of your Stencila document.
Stencila does all this using its [execution engine](computation/engine.md).

The engine manages automatic dependencies between the cells which are specific for each language. For cells written in
R it is farily simple.  If you want to capture the output of the cell,
create a variable and assign (`<-`) the output.
Note that the variables in Stencila are non-mutable :sparkles: . Whatever is on the right hand of the assignment (`<-`)
will become the cell input.

You can the refer to this cell's input and output in the Stencila document.

If you do not capture the output explicitly, you will not be able to refer to it later on. But the input of the cell
will still be available.

For example:

```{r}
x <- 4
sqrt(x)
```

The input for this cell is `x`, the output is empty (`null`) and the value is 2 (`sqrt(4)`).

If you want to caputure the output and be able to refer back to it in the future you need to
modify the cell as follows:

```{r}
x <- 4
y <- sqrt(x)
```

The output is now `y` and you can refer back to this variable in any other cell in Stencila.



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

![Registering R Functions with Stencila Spreadsheet](img/registering-functions.gif)

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

#' Module for defining a list of non-standard evaluation functions,
#' functions that use `substitute()` (or related) on one or more arguments.
#' See http://adv-r.had.co.nz/Computing-on-the-language.html.
#' This list is used when R code is compiled to ignore some
#' variable names when determining the `uses` property
#' of the chunk.

#' Create a entry for a function that uses NSE
#'
#' @param func Name of the function
#' @param package Name of the package that the function is in
#' @param names Names of parameters that should be ignored
#' @param positions Positions of parameters that should be ignored
nse_func <- function(func, package, names=NULL, positions=NULL) {
  list(
    func = func,
    package = package,
    names = names,
    positions = positions
  )
}

#' List of functions that read from files
#' @export
nse_funcs <- list(
  # base package
  nse_func("base", "subset", c("subset", "select"), c(2, 4)),
  # dplyr package
  nse_func("dplyr", "filter")
)

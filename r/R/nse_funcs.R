#' Module for defining a list of non-standard evaluation functions,
#' functions that use `substitute()` (or related) on one or more arguments.
#' See http://adv-r.had.co.nz/Computing-on-the-language.html.
#' This list is used when R code is compiled to ignore some
#' variable names when determining the `uses` property
#' of the chunk.
#'
#' @include util.R
NULL

#' Create a entry for a function that uses NSE
#'
#' All arguments are ignored unless `names` or
#' `positions  is specified.
#'
#' @param package Name of the package that the function is in
#' @param func Name of the function
#' @param names Names of parameters that should be ignored
#' @param positions Positions of parameters that should be ignored
nse_func <- function(package, func, names=NULL, positions=NULL) {
  list(
    package = package,
    func = func,
    names = names,
    positions = positions
  )
}

#' List of functions that read from files
#' @export
nse_funcs <- list(
  # base package
  nse_func("base", "~"),
  nse_func("base", "subset", c("subset", "select"), c(2, 4)),
  # dplyr package
  nse_func("dplyr", "filter"),
  nse_func("dplyr", "arrange"),
  nse_func("dplyr", "select"),
  nse_func("dplyr", "rename"),
  nse_func("dplyr", "mutate"),
  nse_func("dplyr", "transmute"),
  nse_func("dplyr", "summarise")
)

#' List of possible function call names to match
nse_funcs_names <- reduce(
  nse_funcs,
  function(prev, curr) c(prev, curr$func, paste0(curr$package, "::", curr$func)),
  character()
)

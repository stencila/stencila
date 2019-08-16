#' Module for defining a list of "read" functions, functions
#' that read files from the filesystem. This list is used
#' when R code is compiled to help determine the `reads` propery
#' of the chunk

#' Create a entry for a function that reads a file
#'
#' Most file reading functions have the file path as their first
#' parameter named `file`. If this is not the case, or if there
#' is more than one parameter that relates to a file that is read
#' by the function, use the `names` and `positions` parameters
#'
#' @param package Name of the package that the function is in
#' @param func Name of the function
#' @param names Names of parameters that are file paths that are read
#' @param positions Positions of parameters that are file paths that are read
read_func <- function(package, func, names="file", positions=1) {
  list(
    package = package,
    func = func,
    names = names,
    positions = positions
  )
}

#' List of functions that read from files
#' @export
read_funcs <- list(
  # utils package
  read_func("utils", "read.table"),
  read_func("utils", "read.csv"),
  read_func("utils", "read.csv2"),
  read_func("utils", "read.delim"),
  read_func("utils", "read.delim2"),
  read_func("utils", "read.fwf"),
  # foreign package
  read_func("foreign", "read.arff"),
  read_func("foreign", "read.dbf"),
  read_func("foreign", "read.dta"),
  read_func("foreign", "read.epiinfo"),
  read_func("foreign", "read.mtp"),
  read_func("foreign", "read.octave"),
  read_func("foreign", "read.spss"),
  read_func("foreign", "read.ssd"),
  read_func("foreign", "read.systat"),
  read_func("foreign", "read.xport")
)

#' List of possible function call names to match
read_funcs_names <- Reduce(
  function(prev, curr) c(prev, curr$func, paste0(curr$package, "::", curr$func)),
  read_funcs,
  character()
)

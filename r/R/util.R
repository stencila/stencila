# Utility functions used internally in this package
# and not intended to be exported.

#' Map a function across entries in an object
#'
#' This is analagous to `Object.entries(object).map(...)`
#' in Javascript. It handles bother scalar and vector
#' object types.
#'
#' @param object The object to map over
#' @param func The function to apply to each of the object's entries
#' @param ... Additional arguments to pass through to the function
map <- function(object, func, ...) {
  if (is.list(object)) lapply(object, func, ...)
  else func(object, ...)
}

#' Create a transformattion of a object by recursively
#' applying a function to it. Could be called `deepMap`.
#'
#' @param object The object to map over
#' @param func The function to apply to each object
#' @param ... Additional arguments to pass through to the function
transform <- function(object, func, ...) {
  map(object, function(child) map(child, func, ...))
}

# Utility functions used internally in this package
# and not intended to be exported. These functions
# provide for some consistency between implementations
# in this packages and those in other languages, in
# particular Javascript / Typescript.

# The following higher order functions provide a
# familiar functional API without depending
# on `purrr` or similar.

#' Filter an object
#'
#' Analagous to `Array.filter` in Javascript
#'
#' @param object The object to filter
#' @param func The predicate function
filter <- function(object, func) Filter(func, object)

#' Map a function across entries in an object
#'
#' This is analagous to `Object.entries(object).map(...)`
#' in Javascript. It handles both scalar and vector
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

#' Redcue
#'
#' Analagous to `Array.reduce` in Javascript
#'
#' @param object The object to filter
#' @param func The reducer function
#' @param init The initial value
reduce <- function(object, func, init) Reduce(func, object, init)

# The following string functions provide simplified string
# handling without depending on `stringr`.

#' Extract regex matches from a string
#'
#' Similar to the `stringr::str_match` and Javascript's `String.match`.
#'
#' @param string The string to match
#' @param regex The regex to match against
#' @return A character vector of the match including groups. NULL if no match.
string_match <- function(string, regex) {
  match <- regmatches(string, regexec(regex, string))[[1]]
  if (length(match) == 0) NULL else match
}

#' Split a string
#'
#' @param string The string to split
#' @param regex The regex to split using
#' @return A character vector.
string_split <- function(string, regex) {
  strsplit(string, regex)[[1]]
}

#' Get characters on the right on a string
#'
#' @param string The string to extract chars from
#' @param chars Number of chars to extract
string_right <- function(string, chars = 1){
  substr(string, nchar(string) - (chars - 1), nchar(string))
}

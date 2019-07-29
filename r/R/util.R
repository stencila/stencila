# Utility functions used internally in this package
# and not intended to be exported.

#' Map a function across entries in a node
#'
#' This is analagous to `Object.entries(node).map(...)`
#' in Javascript. It handles bother scalar and vector
#' node types.
#'
#' @param node The node to map over
#' @param func The function to apply to each of the node's entries
#' @param ... Additional arguments to pass through to the function
map <- function(node, func, ...) {
  if (is.list(node)) lapply(node, func, ...)
  else func(node, ...)
}

#' Transform a node by recursively applying a function to it.
#'
#' @param node The node to map over
#' @param func The function to apply to each node
#' @param ... Additional arguments to pass through to the function
transform <- function (node, func, ...) {
  map(node, function(child) map(child, func, ...))
}

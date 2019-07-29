# Functions for interoperability between Stencila schema nodes and
# R object and JSON.

#' Convert an R object to a node
#'
#' @param obj The object to convert
node_from_object <- function(obj) {
  if (is.list(obj) && !is.null(obj$type) && exists(obj$type)) entity_from_list(obj)
  else obj
}

#' Convert a node to JSON
#'
#' @param node The schema node to convert
#' @param pretty Should the JSON be pretyy printed (indented)
#' @export
node_to_json <- function(node, pretty = FALSE) {
  as.character(
    # Suppress the "collapse=FALSE called for named list." warning
    # TODO: Check that this warning can be ignored
    suppressWarnings(
      jsonlite::toJSON(node, null = "null", na = "null", pretty = pretty)
    )
  )
}

#' Create a node from JSON
#'
#' @param json The JSON to parse
#' @export
node_from_json <- function(json) {
  node_from_object(
    jsonlite::fromJSON(json, simplifyDataFrame = FALSE)
  )
}

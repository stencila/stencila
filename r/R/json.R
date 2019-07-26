#' Convert a node to JSON
#'
#' @param node The schema node to convert
#' @param pretty Should the JSON be pretyy printed (indented)
#' @export
to_json <- function(node, pretty = FALSE) {
  if (inherits(node, "Entity")) {
    node <- c(
      list(type = jsonlite::unbox(last_class(node))),
      node
    )
  }
  as.character(
    jsonlite::toJSON(node, null = "null", na = "null", pretty = pretty)
  )
}

#' Create a node from JSON
#'
#' @param json The JSON to parse
#' @export
from_json <- function(json) {
  obj <- jsonlite::fromJSON(json, simplifyVector = FALSE)
  if (is.list(obj) && !is.null(obj$type)) {
    type <- obj$type
    obj$type <- NULL
    do.call(type, obj)
  } else {
    obj
  }
}

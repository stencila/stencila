# Methods for lists of class `Entity`

#' Print the entity
#'
#' This simply prints the JSON representation
#' of the node.
#' @export
print.Entity <- function(entity) {
  cat(node_to_json(entity, pretty = TRUE)) # nocov
}

#' @export
entity_from_list <- function(list) {
  type <- list$type
  if (is.null(type)) stop("List must have type property")

  # Remove `type` from the object for the call to the
  # constructor function (which does not have `type` as
  # a parameter, but instead adds it).
  list$type <- NULL

  # Recursively call constructors of list children
  list <- transform(list, node_from_object)

  # Call the constructor function for the type
  do.call(type, list)
}

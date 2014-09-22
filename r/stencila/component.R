#' @include stencila.R
NULL

#' The Component class
#'
#' @name Component
#' @aliases Component-class
#' @exportClass Component
#' @export
class_('Component',NULL)
Component <- function() new("Component")

#' Get or set the path of a component
#'
#' @param path New path for the component
#'
#' @name path
#'
#' @docType methods
#' @rdname path-methods
#' @aliases path,Component-method
NULL

attr_('Component','path',toString)

#' Commit a component
#'
#' @param message A message to describe the commit
#'
#' @name commit
#'
#' @docType methods
#' @rdname commit-methods
#' @aliases commit,Component-method
NULL

Component_commit <- function(instance,message=""){
	call_('Component_commit',instance@pointer,toString(message))
}

#' Get commits for component
#'
#' @name commits
#'
#' @docType methods
#' @rdname commits-methods
#' @aliases commits,Component-method
#'
#' @examples
#' # Create a component, commit it and get a list of commits ...
#' c <- Component()
#' c$commit("Initial commit")
#' c$commits()
NULL

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

attr_('Component','title')
attr_('Component','description')
attr_('Component','keywords')
attr_('Component','authors')

#' Get or set the path of a component
#'
#' @param path New path for the component
#'
#' @export
#' @name path
#'
#' @docType methods
#' @rdname path-methods
#' @aliases path,Component-method
NULL

attr_('Component','path',toString)
setGeneric('path',function(instance,path,...) standardGeneric('path'))
setMethod('path','Component',Component_path)

#' Commit a component
#'
#' @param message A message to describe the commit
#'
#' @export
#' @name commit
#'
#' @docType methods
#' @rdname commit-methods
#' @aliases commit,Component-method
NULL

Component_commit <- function(instance,message=""){
	call_('Component_commit',instance@pointer,toString(message))
}
setGeneric('commit',function(instance,message) standardGeneric('commit'))
setMethod('commit','Component',Component_commit)


#' Get history for component
#'
#' @export
#' @name history
#'
#' @docType methods
#' @rdname history-methods
#' @aliases history,Component-method
#'
#' @examples
#' # Create a component, commit it and get a history of commits ...
#' c <- Component()
#' c$commit("Initial commit")
#' c$history()
NULL

setGeneric('history',function(instance) standardGeneric('history'))
setMethod('history','Component',function(instance) instance$history())

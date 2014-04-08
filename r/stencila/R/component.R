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

attr_('Component','path')

Component_commit <- function(instance,message=""){
	call_('Component_commit',instance@pointer,message)
}

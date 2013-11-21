#' @include shortcuts.R
NULL

#' The Tableset class
#'
#' @param uri The unique resource identifier (URI) for the Tableset
#' 
#' @name Tableset
#' @aliases Tableset-class
#' @seealso Tableset-uri Tableset-table Tableset-indices
#' @exportClass Tableset
#' @export
#'
#' @examples
#' # Create a Tableset in memory...
#' ds <- Tableset()
#' # ... or on disk
#' ds <- Tableset("mytableset.sds")
class_('Tableset')
Tableset <- function(uri="") new("Tableset",uri=uri)

#' Get the unique resource identifier (URI) for the Tableset
#'
#' @name Tableset-uri
#' @aliases uri,Tableset-method
#' @export
setGeneric("uri",function(object) standardGeneric("uri"))
setMethod("uri","Tableset",function(object) object$uri())

#' List the tables in the tableset
#'
#' @name Tableset-tables
#' @aliases tables,Tableset-method
#' @export
setGeneric("tables",function(object) standardGeneric("tables"))
setMethod("tables","Tableset",function(object) object$tables())

#' List the indices in the tableset
#'
#' @name Tableset-indices
#' @aliases indices,Tableset-method
#' @export
setGeneric("indices",function(object) standardGeneric("indices"))
setMethod("indices","Tableset",function(object) object$indices())

#' Execute an SQL select statement and returnthe resulting Datatable
#'
#' @name Tableset-select
#' @aliases select,Tableset-method
#' @export
setGeneric("select",function(object,sql) standardGeneric("select"))
setMethod("select","Tableset",function(object,sql) object$select(sql))

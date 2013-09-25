#' @include shortcuts.R
NULL

#' The Dataset class
#'
#' @param uri The unique resource identifier (URI) for the Dataset
#' 
#' @name Dataset
#' @aliases Dataset-class
#' @seealso Dataset-uri Dataset-table Dataset-indices
#' @exportClass Dataset
#' @export
#'
#' @examples
#' # Create a Dataset in memory...
#' ds <- Dataset()
#' # ... or on disk
#' ds <- Dataset("mydataset.sds")
class_('Dataset')
Dataset <- function(uri="") new("Dataset",uri=uri)

#' Get the unique resource identifier (URI) for the Dataset
#'
#' @name Dataset-uri
#' @aliases uri,Dataset-method
#' @export
setGeneric("uri",function(object) standardGeneric("uri"))
setMethod("uri","Dataset",function(object) object$uri())

#' List the tables in the dataset
#'
#' @name Dataset-tables
#' @aliases tables,Dataset-method
#' @export
setGeneric("tables",function(object) standardGeneric("tables"))
setMethod("tables","Dataset",function(object) object$tables())

#' List the indices in the dataset
#'
#' @name Dataset-indices
#' @aliases indices,Dataset-method
#' @export
setGeneric("indices",function(object) standardGeneric("indices"))
setMethod("indices","Dataset",function(object) object$indices())

#' Execute an SQL select statement and returnthe resulting Datatable
#'
#' @name Dataset-select
#' @aliases select,Dataset-method
#' @export
setGeneric("select",function(object,sql) standardGeneric("select"))
setMethod("select","Dataset",function(object,sql) object$select(sql))

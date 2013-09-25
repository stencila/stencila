#' @include shortcuts.R
NULL

#' The Datatable class
#'
#' @name Datatable
#' @aliases Datatable-class
#' @exportClass Datatable
#' @export
#'
#' @examples
#' # Create a Datatable...
#' dt <- Datatable()
#' # which is equivalent to, but a bit quicker to type than,...
#' dt <- new("Datatable")
class_('Datatable')
Datatable <- function(from) {
    if(!missing(from)) return(as.Datatable(from))
    else return(new("Datatable"))
}

#' Get the number of rows in a datatable
#'
#' @name Datatable-rows
#' @aliases rows,Datatable-method
#' @export
setGeneric("rows",function(object) standardGeneric("rows"))
setMethod("rows","Datatable",function(object) object$rows())

#' Get the number of columns in a datatable
#'
#' @name Datatable-columns
#' @aliases columns,Datatable-method
#' @export
setGeneric("columns",function(object) standardGeneric("columns"))
setMethod("columns","Datatable",function(object) object$columns())

#' Get the dimensions (rows x columns) of a datatable
#'
#' @name Datatable-dimensions
#' @aliases dimensions,Datatable-method
#' @export
setGeneric("dimensions",function(object) standardGeneric("dimensions"))
setMethod("dimensions","Datatable",function(object) object$dimensions())

#' Get the name of a column of a datatable
#'
#' Note that this method corresponds to the "name" method in the Stencila C++ package.
#' However, that name for a method appear to cause problems in R so for the R package we have used "colname"
#'
#' @name Datatable-colname
#' @aliases colname,Datatable,integer-method
#' @export
setGeneric("colname",function(object,column) standardGeneric("colname"))
setMethod("colname",c("Datatable","integer"),function(object,column) object$colname(column))

#' Get the names of the columns of a datatable
#'
#' @name Datatable-colnames
#' @aliases colnames,Datatable-method
#' @export
setGeneric("colnames",function(x) standardGeneric("colnames"))
setMethod("colnames","Datatable",function(x) x$colnames())

#' Get the type of a column of a datatable
#'
#' @name Datatable-type
#' @aliases type,Datatable,integer-method
#' @export
setGeneric("type",function(object,column) standardGeneric("type"))
setMethod("type",c("Datatable","integer"),function(object,column) object$type(column))

#' Get the types of the columns of a datatable
#'
#' @name Datatable-types
#' @aliases types,Datatable-method
#' @export
setGeneric("types",function(object) standardGeneric("types"))
setMethod("types","Datatable",function(object) object$types())

#' Create an index for a datatable based on one or more columns
#'
#' @name Datatable-index
#' @aliases index,Datatable-method
#' @export
setGeneric("index",function(object,columns) standardGeneric("index"))
setMethod("index","Datatable",function(object,columns) object$types(columns))

#' Get a list of indices created on a datatable
#'
#' @name Datatable-indices
#' @aliases indices,Datatable-method
#' @export
setGeneric("indices",function(object) standardGeneric("indices"))
setMethod("indices","Datatable",function(object) object$indices())

Datatable_head <- function(self,rows=10) return(object_(call_('Datatable_head',self@pointer,rows)))
Datatable_tail <- function(self,rows=10) return(object_(call_('Datatable_tail',self@pointer,rows)))
Datatable_value <- function(self,row=0,col=0) return(object_(call_('Datatable_value',self@pointer,row,col)))

# Replicating S3 methods for data.frames
# See
#   methods(class='data.frame')
# for a full list

setMethod("show", "Datatable", function(object){
    cat("Datatable (rows:",object$rows(),", columns:",object$columns(),")\n",sep="")
    print(as.data.frame(object$head(100)))
    if(object$rows()>100) cat('...<truncated>...\n')
})

#' Datatable subscript
#'
#' @name Datatable-subscript
#' @aliases [,Datatable-method
#' @rdname Datatable-subscript
setMethod('[',
          signature(x='Datatable'),
          function(x,i,j,...){
              #Dispatch needs to be done here rather than using several alternative
              #signatures in setMethod dispatch. That is because if the latter is used
              #then evaluation of arguments is done in the parent frame and expressions
              #such as "by(year)" fail
              #    i='missing',j='numeric': get column(s)
              #	i='missing',j='character' : get column(s)
              #	i='numeric',j='missing' : get row(s)
              #	i='numeric',j='numeric' : get values(s)
              #	i='numeric',j='character' : get values(s)
              
              #Record the call, removing first (name of function ('[')) and second ('x') arguments which are
              #not needed
              args <- as.list(match.call()[-c(1,2)])
              
              rows <- NULL
              cols <- NULL
              directives <- NULL
              
              # Intialise a list of names that refer to column in the Datatable
              # or functions in the dataset. Other names will be searched for in the 
              # R parent frame
              datatable_names <- dataquery_elements_
              for(name in x$colnames()){
                  datatable_names[[name]] <- Column(name)
              }
              
              for(index in 1:length(args)){
                  arg <- args[[index]]
                  arg <- substitute(arg)
                  name <- names(args)[[index]]
                  
                  directive <- NULL
                  
                  if(mode(arg)=='numeric'){
                      if(name=='i') rows = eval(arg)
                      else if(name=='j') cols = eval(arg)
                      else {
                          directive = paste(directive," const(",arg,")",sep='')
                      }
                  }
                  else if(mode(arg)=='call'){
                      # Evaluate each argument with the parent frame as a "fallback" for symbols not in the database
                      # See subset.data.frame for an example of this
                      directive <- tryCatch(eval(arg,datatable_names,parent.frame()),error=function(error) error)
                      if(inherits(directive,'error')){
                          stop(paste("in query :",directive$message,sep=''),call.=FALSE)
                      }
                  }
                  if(!is.null(directive)) directives <- c(directives,directive)
              }
              
              if(!is.null(rows) & !is.null(cols)) return(x$value(rows,cols))
              
              query <- do.call(Dataquery,directives)
              return(query$execute(x@pointer))
          }
)

#' Get the dimensions of a Datatable
#'
#' @param x The datatable
#' 
#' @method dim Datatable
#' @name Datatable-dim
#' @aliases dim.Datatable
#' @export
#' 
#' @examples
#' # Create a datatable and get it's dimensions
#' dt <- Datatable()
#' dim(dt)
#' # Implementing dim also provides for nrow and ncol
#' nrow(dt)
#' ncol(dt)
dim.Datatable <- function(x) x$dimensions()

#' Get the first n rows of a Datatable
#'
#' @param x The datatable
#' @param n The number of rows to get
#' 
#' @method head Datatable
#' @name Datatable-head
#' @aliases head.Datatable
#' @export
head.Datatable <- function(x,n=10) as.data.frame(x$head(n))

#' Get the last n rows of a Datatable
#'
#' @param x The datatable
#' @param n The number of rows to get
#' 
#' @method tail Datatable
#' @name Datatable-tail
#' @aliases tail.Datatable
#' @export
tail.Datatable <- function(x,n=10) as.data.frame(x$tail(n))

#' Convert a Datatable to a data.frame
#'
#' @param x The datatable
#' 
#' @method as.data.frame Datatable
#' @name Datatable-as.data.frame
#' @aliases as.data.frame.Datatable
#' @export
as.data.frame.Datatable <- function(x, row.names, optional, ...) x$to_dataframe()

#' Convert a data.frame to a Datatable
#'
#'
#' @name Datatable-from-data.frame
#' @aliases as.Datatable,data.frame-method
#' @export
setGeneric("as.Datatable",function(object) standardGeneric("as.Datatable"))
setMethod("as.Datatable","data.frame",function(object){
    # Currently it is necessary to convert all columns to string type before calling C++ function
    for(name in names(object)) object[,name] = as.character(object[,name])
    create_("Datatable","Datatable_from_dataframe",object)
})

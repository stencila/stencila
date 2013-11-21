#' @include shortcuts.R
NULL

#' The Table class
#'
#' @name Table
#' @aliases Table-class
#' @exportClass Table
#' @export
#'
#' @examples
#' # Create a Table...
#' dt <- Table()
#' # which is equivalent to, but a bit quicker to type than,...
#' dt <- new("Table")
class_('Table')
Table <- function(from) {
    if(!missing(from)) return(as.Table(from))
    else return(new("Table"))
}

#' Get the number of rows in a table
#'
#' @name Table-rows
#' @aliases rows,Table-method
#' @export
setGeneric("rows",function(object) standardGeneric("rows"))
setMethod("rows","Table",function(object) object$rows())

#' Get the number of columns in a table
#'
#' @name Table-columns
#' @aliases columns,Table-method
#' @export
setGeneric("columns",function(object) standardGeneric("columns"))
setMethod("columns","Table",function(object) object$columns())

#' Get the dimensions (rows x columns) of a table
#'
#' @name Table-dimensions
#' @aliases dimensions,Table-method
#' @export
setGeneric("dimensions",function(object) standardGeneric("dimensions"))
setMethod("dimensions","Table",function(object) object$dimensions())

#' Get the label for a column in a table
#'
#'
#' @name Table-label
#' @aliases label,Table,integer-method
#' @export
setGeneric("label",function(object,column) standardGeneric("label"))
setMethod("label",c("Table","integer"),function(object,column) object$label(column))

#' Get the labels of all the columns in a table
#'
#' @name Table-labels
#' @aliases labels,Table-method
#' @export
setGeneric("labels",function(x) standardGeneric("labels"))
setMethod("labels","Table",function(x) x$labels())

#' Get the type of a column of a table
#'
#' @name Table-type
#' @aliases type,Table,integer-method
#' @export
setGeneric("type",function(object,column) standardGeneric("type"))
setMethod("type",c("Table","integer"),function(object,column) object$type(column))

#' Get the types of the columns of a table
#'
#' @name Table-types
#' @aliases types,Table-method
#' @export
setGeneric("types",function(object) standardGeneric("types"))
setMethod("types","Table",function(object) object$types())

#' Create an index for a table based on one or more columns
#'
#' @name Table-index
#' @aliases index,Table-method
#' @export
setGeneric("index",function(object,columns) standardGeneric("index"))
setMethod("index","Table",function(object,columns) object$types(columns))

#' Get a list of indices created on a table
#'
#' @name Table-indices
#' @aliases indices,Table-method
#' @export
setGeneric("indices",function(object) standardGeneric("indices"))
setMethod("indices","Table",function(object) object$indices())

Table_head <- function(self,rows=10) return(object_(call_('Table_head',self@pointer,rows)))
Table_tail <- function(self,rows=10) return(object_(call_('Table_tail',self@pointer,rows)))
Table_value <- function(self,row=0,col=0) return(object_(call_('Table_value',self@pointer,row,col)))

# Replicating S3 methods for data.frames
# See
#   methods(class='data.frame')
# for a full list

setMethod("show", "Table", function(object){
    cat("Table (rows:",object$rows(),", columns:",object$columns(),")\n",sep="")
    print(as.data.frame(object$head(100)))
    if(object$rows()>100) cat('...<truncated>...\n')
})

#' Table subscript
#'
#' @name Table-subscript
#' @aliases [,Table-method
#' @rdname Table-subscript
setMethod('[',
          signature(x='Table'),
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
              
              # Intialise a list of names that refer to column in the Table
              # or functions in the Tableset. Other names will be searched for in the 
              # R parent frame
              table_names <- query_elements_
              for(label in x$labels()){
                  table_names[[name]] <- Column(label)
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
                      directive <- tryCatch(eval(arg,table_names,parent.frame()),error=function(error) error)
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

#' Get the dimensions of a Table
#'
#' @param x The table
#' 
#' @method dim Table
#' @name Table-dim
#' @aliases dim.Table
#' @export
#' 
#' @examples
#' # Create a table and get it's dimensions
#' dt <- Table()
#' dim(dt)
#' # Implementing dim also provides for nrow and ncol
#' nrow(dt)
#' ncol(dt)
dim.Table <- function(x) x$dimensions()

#' Get the first n rows of a Table
#'
#' @param x The table
#' @param n The number of rows to get
#' 
#' @method head Table
#' @name Table-head
#' @aliases head.Table
#' @export
head.Table <- function(x,n=10) as.data.frame(x$head(n))

#' Get the last n rows of a Table
#'
#' @param x The table
#' @param n The number of rows to get
#' 
#' @method tail Table
#' @name Table-tail
#' @aliases tail.Table
#' @export
tail.Table <- function(x,n=10) as.data.frame(x$tail(n))

#' Convert a Table to a data.frame
#'
#' @param x The table
#' 
#' @method as.data.frame Table
#' @name Table-as.data.frame
#' @aliases as.data.frame.Table
#' @export
as.data.frame.Table <- function(x, row.names, optional, ...) x$to_dataframe()

#' Convert a data.frame to a Table
#'
#'
#' @name Table-from-data.frame
#' @aliases as.Table,data.frame-method
#' @export
setGeneric("as.Table",function(object) standardGeneric("as.Table"))
setMethod("as.Table","data.frame",function(object){
    # Currently it is necessary to convert all columns to string type before calling C++ function
    for(name in names(object)) object[,name] = as.character(object[,name])
    create_("Table","Table_from_dataframe",object)
})

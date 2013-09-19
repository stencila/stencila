#' Stencila R package
#'
#' @docType package
#' @name stencila
#' @aliases stencila stencila-package
#' @author Nokome Bentley <nokome.bentley@@stenci.la>
#' @useDynLib stencila_
#' @import utils
NULL

########################################################################
# Convienience functions used internally in the Stencila package and
# not exported
########################################################################

# A convienience function for calling C++ functions in 
# the Stencila R extension module
call_ <- function(symbol,...){
    .Call(symbol,...,package='stencila')
}

# A convienience function for converting an externalptr object
# into an instance of a Stencila R class
object_ <- function(object){
  if(typeof(object)=='externalptr'){
    class <- call_("Stencila_class",object)
    object <- new(class,created=TRUE,pointer=object)
    return(object)
  }
  return(object)
}

# A convienience function for creating an instance of a Stencila R class
# from a function other than "<class>_new"
create_ <- function(class,func,...){
  new(class,created=TRUE,pointer=call_(func,...))
}

# A convienience function for calling C++ functions which 
# represent Stencila R class methods
method_ <- function(class_name,method_name,...){
    symbol <- paste(class_name,method_name,sep='_')
    # Look for a R version of symbol and call it, otherwise
    # try to get it from C++ symbols 
    if(exists(symbol)){
      return(get(symbol)(...))
    } else {
      result <- tryCatch(
          call_(symbol,...),
          error = function(error) error
      )
      if(inherits(result,'error')){
          if(result$message==paste('C symbol name "',symbol,'" not in load table',sep='')){
              stop(paste('Class ',class_name,' does not have a method named "',method_name,'"',sep=''),call.=FALSE)
          } else {
              stop(result$message,call.=FALSE)
          }
      }
    }
    result
}

# A convienience function for creating a Stencila R class
class_ <- function(class_name){
  
  setClass(
      class_name,
      representation=representation(
        created = 'logical',
        pointer = 'externalptr'
      ),
      prototype=prototype(
        created = FALSE
      )     
  )

  setMethod('initialize',class_name,eval(substitute(function(.Object,created=FALSE,pointer=NULL,...){
    if(!created){
      .Object@pointer = method_(class_name,'new',...)
      .Object@created = TRUE
    } else {
      .Object@pointer = pointer
      .Object@created = TRUE
    }
    return(.Object)
  },list(class_name=class_name))))
  
  setMethod('$',class_name,eval(substitute(function(x,name){
    function(...) {
      result <- method_(class_name,name,x@pointer,...)
      #If the return is NULL (in C++ nil) then return self
      #so method chaining can be used...
      if(is.null(result)) return(x)
      #...otherwise return the object after wrapping it
      else return(object_(result))
      #We could just get C++ functions to return self
      #and wrap the return regardless of type but that creates a new object
      #and would seem to be wateful (and perhaps dangerous?)
    }
  },list(class_name=class_name))))
  
  NULL
}

########################################################################
# Package startup/shutdown functions
#
# See ?.onLoad
########################################################################
.onLoad <- function(libname, pkgname){
  call_('Stencila_startup')
}

.onUnload <- function(libpath){
}

########################################################################
# Utility functions
########################################################################

#' Get the version of the Stencila R package
#'
#' @export
#' @examples
#'   stencila::version()
version <- function(){
  call_('Stencila_version')
}

#' Create an iterator for an R object
#'
#' An iterator is a "helper" object for traversing a container object.
#' Iterators are commonly used in languages such as C++ and Java.
#' Stencila requires iterators for rendering stencils.
#' Specifically, when rendering "each" elements, iterators allow for looping over item in a container
#' using a consistent interface: one which exposes the $step() and $more() methods.
#' There is already an 'iterator' package for R.
#' We have not used that package here because Stencila requires fairly simple iterator functionality and
#' it would introduce another dependency.
#' However, users looking for more advanced and comprehensive iterators
#' should consider using the 'iterator' package.
#'
#' @param container The container object to iterate over
#' 
#' @export
#'
#' @examples
#' i <- iterate(c(1,2,3))
#' i$step() # -> 1
#' i$step() # -> 2
#' i$more() # -> TRUE
#' i$step() # -> 3
#' i$more() # -> FALSE
iterate <- function(container){
  UseMethod('iterate')
}

# A default iterator for vectors and lists
DefaultIterator <- function(container){
  self <- new.env()
  class(self) <- "DefaultIterator"
  
  self$container <- container
  self$index <- 0
  self$size <- length(self$container)
  
  self$more <- function(){
    return(self$index<self$size)
  }
  
  self$step <- function(){
    self$index <- self$index +1
    return(self$container[[self$index]])
  }
  
  self
}
#' @export
iterate.default <- function(container){
  return(DefaultIterator(container))
}

# Other iterate methods will be added when
# other iterator classes are written (e.g. DataframeIterator, MatrixIterator)

########################################################################
# HttpServer
########################################################################

#' The HttpServer class
#'
#' @name HttpServer
#' @aliases HttpServer-class
#' @seealso HttpServer-start HttpServer-stop HttpServer-run
#' @exportClass HttpServer
#' @export
#'
#' @examples
#' # Create a HTTP server...
#' server <- HttpServer()
class_('HttpServer')
HttpServer <- function() new("HttpServer")


########################################################################
# Dataset
########################################################################

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

########################################################################
# Datacursor
########################################################################

#' The Datacursor class
#'
#' @name Datacursor
#' @aliases Datacursor-class
#' @exportClass Datacursor
class_('Datacursor')

########################################################################
# Datatable
########################################################################

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

Datatable_head <- function(self,rows=10) return(object_(call_('Datatable_head',self,rows)))
Datatable_tail <- function(self,rows=10) return(object_(call_('Datatable_tail',self,rows)))
Datatable_value <- function(self,row=0,col=0) return(object_(call_('Datatable_value',self,row,col)))

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
    #	i='missing',j='numeric': get column(s)
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

########################################################################
# Dataquery and elements
########################################################################

class_('Element')

# Create an expression element
elem_ <- function(class,...){
  create_('Element',paste('Element_',class,sep=''),...)
}

#' Create a data query constant
#' 
#' This function is exported mainly for use in testing data queries and is unlikely to be
#' used directly by the user
#' 
#' @param value The value of the constant
#' @export
Constant <- function(value) const_(value)

# Convert fundamental types to a
const_ <- function(object){
  if(inherits(object,'logical')) return(elem_('Logical',object))
  if(inherits(object,'integer')) return(elem_('Integer',object))
  if(inherits(object,'numeric')) return(elem_('Numeric',object))
  if(inherits(object,'character')) return(elem_('String',object))
  stop(paste("Object of class",paste(class(object),collapse=","),"is not a convertable to a Constant"),call.=FALSE)
}

wrap_ <- function(object){
  if(is.null(object)) return(call_("Element_null"))
  
  if(inherits(object,'Element'))  return(object@pointer)
  
  # Attempt a conversion to a Constant
  const = tryCatch(const_(object),error=function(error)error)
  if(!inherits(const,'error')) return(const@pointer)
  
  # If got to here then raise an error
  stop(paste("Object of class",paste(class(object),collapse=","),"is not wrappable"),call.=FALSE)
}

wrap_all_ <- function(...){
  #Create a list from arguments
  args <- list(...)
  #Create a vector of Dataquery elements by wrapping each argument
  elements <- vector()
  n <- length(args)
  if(n>0){
    for(index in 1:n) elements <- c(elements,wrap_(args[[index]]))
  }
  elements
}

#' Create a Datatable column identifier
#' 
#' This function is exported mainly for use in testing data queries and is unlikely to be
#' used directly by the user
#' 
#' @param name Name of the column
#' @export
Column <- function(name) elem_('Column',name)

# Unary operators
# When writing these, use getMethod() to obtain the correct function argument names e.g.
#   getMethod("==")
# Note that S4 method aliases are provided to prevent warning from R CMD check
unop_ <- function(class,expr){
  elem_(class,wrap_(expr))
}

#' Operators for Dataquery elements
#'
#' @name Element-operators
#' @aliases +,Element,missing-method -,Element,missing-method !,Element-method
setMethod("+",signature(e1='Element',e2='missing'),function(e1,e2) unop_('Positive',e1))
setMethod("-",signature(e1='Element',e2='missing'),function(e1,e2) unop_('Negative',e1))
setMethod("!",signature(x='Element'),function(x) unop_('Not',x))

# Binary operators
# When writing these, use getMethod() to obtain the correct function argument names e.g.
#   getMethod("==")

# For each binary operator, set a method with an Element on both sides          
binop_ <- function(class,left,right){
  elem_(class,wrap_(left),wrap_(right))
}

binop_methods_ <- function(op,name){
  setMethod(op,signature(e1='Element'),function(e1,e2) binop_(name,e1,e2))
  setMethod(op,signature(e2='Element'),function(e1,e2) binop_(name,e1,e2))
  setMethod(op,signature(e1='Element',e2='Element'),function(e1,e2) binop_(name,e1,e2))
}

binop_methods_('*','Multiply')
binop_methods_('/','Divide')
binop_methods_('+','Add')
binop_methods_('-','Subtract')

binop_methods_('==','Equal')
binop_methods_('!=','NotEqual')
binop_methods_('<','LessThan')
binop_methods_('<=','LessEqual')
binop_methods_('>','GreaterThan')
binop_methods_('>=','GreaterEqual')

binop_methods_('&','And')
binop_methods_('|','Or')

#' @export
setMethod('%in%',signature(x='Element',table='ANY'),function(x,table){
  elem_('In',wrap_(x),table)
})

#' @export
Call <- function(name,...) elem_('Call',name,wrap_all_(...))

#' @export
Aggregate <- function(name,element) elem_('Aggregate',name,wrap_(element))

#' @export
As <- function(name,element) elem_('As',name,wrap_(element))

#' @export
Distinct <- function() elem_('Distinct')

#' @export
All <- function() elem_('All')

#' @export
Where <- function(element) elem_('Where',wrap_(element))

#' @export
By <- function(element) elem_('By',wrap_(element))

#' @export
Having <- function(element) elem_('Having',wrap_(element))

#' @export
Order <- function(element) elem_('Order',wrap_(element))

#' @export
Limit <- function(number) elem_('Limit',number)

#' @export
Offset <- function(number) elem_('Offset',number)

#' @export
Top <- function(by,element,number) elem_('Top',wrap_(by),wrap_(element),number)

#' @export
Margin <- function(by=NULL) elem_('Margin',wrap_(by))

#' @export
Proportion <- function(value,by=NULL) elem_('Proportion',wrap_(value),wrap_(by))

# Dataquery elements
# This list is used in Datable subscript operator to provide
# a list of available names
dataquery_elements_ <- list(
  distinct = Distinct,
  all = All,
  as = As,
  where = Where,
  by = By,
  having = Having,
  order = Order,
  limit = Limit,
  offset = Offset,
  
  top = Top,
  
  margin = Margin,
  
  prop = Proportion
)
#Fuction calls
func_ <- function(name) eval(substitute(function(...) Call(name,...),list(name=name)))
for(name in c(
  #Math functions
  'cos','sin','tan',
  'acos','asin','atan',  
  'cosh','sinh','tanh',
  'pi','degrees','radians',
  'exp','log','log10',
  'pow','square','sqrt',
  'abs','round','sign','ceil','floor',
  'random'
)) dataquery_elements_[[name]] <- func_(name)

#Aggregators
agg_ <- function(name) eval(substitute(function(...) Aggregate(name,...),list(name=name)))
for(name in c(
  'count','sum','min','max',
  'avg','mean','geomean','harmean',
  'var','sd'
)) dataquery_elements_[[name]] <- agg_(name)

#' The Dataquery class
#'
#' @param ... A set of dataquery elements
#'
#' @name Dataquery
#' @aliases Dataquery-class
#' @exportClass Dataquery
#' @export
class_('Dataquery')
Dataquery <- function(...) {
  # The C++ function Dataquery_new causes a segfault when compiled with g++ -O2
  # and called with no elements. This method dispatches to alternative versions of 
  # of a Dataquery constructor which does not expect arguments. But even that does not
  # work. That code is retained but now this function stops is there are no arguments
  if(length(list(...))==0) {
    stop("a Dataquery must be constructed with at least one argument")
    create_("Dataquery","Dataquery_new_noargs")
  }
  else new("Dataquery",elements=wrap_all_(...))
}

########################################################################
# Stencil
########################################################################

#' The Stencil class
#'
#' Use this function to create a stencil, optionally
#' including any initial content.
#'
#' @param content Initial HTML content of the stencil
#'
#' @name Stencil
#' @aliases Stencil-class
#' @exportClass Stencil
#' @export
#'
#' @examples
#' # Create a Stencil...
#' stencil <- Stencil()
#' # ... which is equivalent to
#' stencil <- new("Stencil")
#' # Create a Stencil and set its content
#' stencil <- Stencil('Pi has a value of: <span data-text="pi"/>')
#' # ... which is equivalent to
#' stencil <- Stencil()
#' stencil$load('Pi has a value of: <span data-text="pi"/>')
class_('Stencil')
Stencil <- function(content) {
  stencil <- new("Stencil")
  if(!missing(content)){
    stencil$load(content)
  }
  return(stencil)
}

#' Load content into a stencil
#'
#' @name Stencil-load
#' @aliases load,Stencil-method
#' @export
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... and load some HTML into this stencil
#' stencil$load("<p>Hello world!</p>")
#' # ... or, equivalently
#' load(stencil,"<p>Hello world!</p>")
setGeneric("load",function(object,content) standardGeneric("load"))
setMethod("load",c("Stencil","ANY"),function(object,content) object$load(content))

#' Render a stencil object or a stencil string 
#'
#' This is a convienience function for creating, rendering and then
#' dumping a stencil.
#' It is useful for quickly executing these three common tasks in stencil usage.
#'
#' @name Stencil-render
#' @aliases render,Stencil-method render,ANY-method
#' @export
setGeneric("render",function(stencil,workspace) standardGeneric("render"))
setMethod("render",c("ANY","ANY"),function(stencil,workspace){
  if(!('Stencil' %in% class(stencil))){
    stencil <- Stencil(stencil)
  }
  
  if(missing(workspace)) workspace <- Workspace(parent.frame())
  else {
    if(!('Workspace' %in% class(workspace))) workspace <- Workspace(workspace)
  }
  
  stencil$render(workspace)
  return(stencil$dump())
})

#' Create a stencil rendering workspace
#' 
#' Stencils are rendered within a workspace. 
#' The workspace determines the variables that are available to the stencil.
#' Often, stencils will be rendered within the workspace of the R global environment.
#' However, if you want to create a different workspace then use this function
#' 
#' @param envir The environment for the workspace. Optional.
#'
#' @export
Workspace <- function(envir){
  
  self <- new.env()
  class(self) <- "Workspace"
  
  if(missing(envir)) envir <- new.env(parent=baseenv())
  else if(inherits(envir,'environment')) envir <- envir
  else if(is.list(envir)) envir <- list2env(envir,parent=baseenv())
  else if(is.atomic(envir) & inherits(envir,'character')){
    if(envir==".") envir <- parent.frame()
    else stop(paste('unrecognised environment flag:',envir))
  }
  else stop(paste('unrecognised environment class:',paste(class(envir),collapse=",")))
  self$stack <- list(envir)
  
  ##################################
  
  self$read_from <- function(dir){
    base::load(paste(dir,'.RData',sep='/'),envir=self$bottom())
  }
  
  self$write_to <- function(dir){
    envir <- self$bottom()
    objs <-ls(envir)
    save(list=objs,envir=envir,file=paste(dir,'.RData',sep='/'))
  }
  
  ##################################
  
  self$push <- function(item){
    self$stack[[length(self$stack)+1]] <- item
    return(self)
  }
  
  self$pop  <- function() {
    self$stack[[length(self$stack)]] <- NULL
    return(self)
  }
  
  self$bottom  <- function() {
    return(self$stack[[1]])
  }
  
  self$top  <- function() {
    return(self$stack[[length(self$stack)]])
  }
  
  ##################################
  
  self$get <- function(expression) {
    return(eval(parse(text=expression),envir=self$top()))
  }
  
  self$set <- function(name,expression){
    env <- self$top()
    value <- eval(parse(text=expression),envir=env)
    assign(name,value,envir=env)
    return(self)
  }
  
  ##################################
  # "script" elements
  # 
  # Executes the script
  self$script <- function(code){
    eval(parse(text=code),envir=self$top())
    return(self)
  }
   
  ##################################
  # "interact" method
  
  self$interact_code <- ""
  
  self$interact <- function(code){
    self$interact_code <- paste(self$interact_code,code,sep="")
    expr <- tryCatch(parse(text=self$interact_code),error=function(error)error)
    if(inherits(expr,'error')){
      if(grepl('unexpected end of input',expr$message)){
        return(paste("C",self$interact_code,sep=""))
      } else {
        self$interact_code <- ""
        return(paste("S",expr$message,sep=""))
      }
    } else {
      self$interact_code <- ""
      result <- tryCatch(eval(expr,envir=self$top()),error=function(error)error)
      if(inherits(expr,'result')){
        return(paste("E",result$message,sep=""))
      } else {
        # show() and capture.output() actually return vectors of strings for each line
        # so they need to be collapsed...
        string <- paste(capture.output(show(result)),collapse='\n')
        return(paste("R",string,sep=""))
      }
    }
  }
  
  ##################################
  # "text" elements
  # 
  # Returns a text representation of the object
  
  self$text <- function(expression){
    value <- self$get(expression)
    stream <- textConnection("text", "w")
    cat(paste(value,collapse=", "),file=stream)
    close(stream)
    return(text)
  }
  
  ##################################
  # "image" elements
  # 
  # Creates a new graphics device then executes the code
  # (which is expected to write to the device) and then 
  # returns the additional nodes.
  self$image_begin <- function(type){
      if(type!='svg') stop(paste('Image type not supported:',type))
      # Create a temporary filename
      filename = tempfile()
      # Create an SVG graphics device
      svg(filename)
      # Store that filename for later
      assign('_svg_',filename,envir=self$top())
  }
  
  self$image_end  <- function(){
      # Get filename
      filename <- base::get('_svg_',envir=self$top())
      # Close all graphics devices. In case the
      # code opened new ones, we really need to closethem all
      graphics.off()
      # Determine file size so we know how many bytes to read in
      bytes = file.info(filename)$size
      # Read in the SVG file and return it
      svg = readChar(filename,nchars=bytes)
      return(svg)
  }
  
  ##################################
  # "if" elements
  # 
  # Returns a boolean evaluation of expression
  
  self$test <- function(expression){
    value <- self$get(expression)
    return(as.logical(value))
  }

  ##################################
  # "switch" elements
  
  self$subject <- function(expression){
    value <- self$get(expression)
    assign('_subject_',value,envir=self$top())
    return(self)
  }
  
  self$match <- function(expression){
    value <- self$get(expression)
    subject <- base::get('_subject_',envir=self$top())
    return(value==subject)
  }
  
  ##################################
  # "with" elements
  #
  # Call enter('expression') at start of a "with" element
  # Call enter() at start of a "block" element
  # Call exit() at the end of a "with" or "block" element
  #
  # See http://digitheadslabnotebook.blogspot.co.nz/2011/06/environments-in-r.html (and links therein)
  # for a useful explanation of environments
  
  self$enter <- function(expression=NULL){
    parent <- self$top()
    if(is.null(expression)) env <- new.env(parent=parent)
    else env <- list2env(self$get(expression),parent=parent) #Use list2env rather than as.enviroment because it allows use to define parent
    self$push(env)
    return(self)
  }
  
  self$exit <- function(){
    self$pop()
    return(self)
  }
  
  ##################################
  # "each" elements
  #
  # Call begin('item','items') at start of an "each" element
  # Call step() at end of each element
  
  self$begin <- function(item,items){
    # Enter a new anonymous block that forms the namespace for the loop
    loop <- self$enter()$top()
    # Create an iterator for items
    items <- eval(parse(text=items),envir=loop)
    iterator <- iterate(items)
    assign('_items_',iterator,envir=loop)
    # Assign special variables into the loop namespace
    # so that when the step() method is called it knows which variables to get
    # and which to set
    assign('_item_',item,envir=loop)
    assign(item,iterator$step(),envir=loop)
    # Return flag indicating if any items
    return(iterator$more())
  }
  
  self$step <- function(){
    loop <- self$top()
    # Get _items_
    items <- base::get('_items_',envir=loop)
    # Go to next item
    if(items$more()){
      item <- items$step()
      assign(
        base::get('_item_',envir=loop),
        item,
        envir=loop
      )
      return(TRUE)
    } else {
      #No more items so exit the loop
      self$exit()
      return(FALSE)
    }
  }
  
  return(self)
}

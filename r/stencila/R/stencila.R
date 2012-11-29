#' Stencila R package
#'
#' @docType package
#' @name stencila
#' @aliases stencila stencila-package
#' @author Nokome Bentley <nokome.bentley@@stenci.la>
#' @useDynLib stencila_
NULL

# A convienience function for calling C++ functions in 
# the Stencila R extension module
call_ = function(symbol,...){
    .Call(symbol,...,package='stencila')
}

# A convienience function for converting an externalptr object
# into an instance of a Stencila R class
object_ = function(object){
  if(typeof(object)=='externalptr'){
    class = call_("Stencila_class",object)
    object = new(class,created=TRUE,pointer=object)
    return(object)
  }
  return(object)
}

# A convienience function for creating an instance of a Stencila R class
# from a function other than "<class>_new"
create_ = function(class,func,...){
  new(class,created=TRUE,pointer=call_(func,...))
}

# A convienience function for calling C++ functions which 
# represent Stencila R class methods
method_ = function(class_name,method_name,...){
    symbol = paste(class_name,method_name,sep='_')
    result = tryCatch(
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
    result
}

# A convienience function for creating a Stencila R class
class_ = function(class_name){
  
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
      result = method_(class_name,name,x@pointer,...)
      if(!is.null(result)) return(object_(result))
    }
  },list(class_name=class_name))))
  
  NULL
}

#####################################

#' Stencila version
#'
#' @export
#' @examples
#'   stencila::version()
version <- function(){
  call_('Stencila_version')
}

#####################################

#' Datacursor
#'
#' @name Datacursor-class
#' @rdname Datacursor-class
#' @exportClass Datacursor
class_('Datacursor')

#####################################

#' Dataset
#'
#' @name Dataset-class
#' @rdname Dataset-class
#' @exportClass Dataset
class_('Dataset')

#' Create a dataset
#'
#' @export
#'
#' @examples
#' # Create a Dataset...
#' ds = Dataset()
#' # which is equivalent to, but a bit quicker to type than,...
#' ds = new("Dataset")
Dataset = function(uri="") new("Dataset",uri=uri)

#####################################

#' The Datatable class
#'
#' @name Datatable-class
#' @rdname Datatable-class
#' @exportClass Datatable
class_('Datatable')

#' Create a datatable
#'
#' @export
#'
#' @examples
#' # Create a Datatable...
#' dt = Datatable()
#' # which is equivalent to, but a bit quicker to type than,...
#' dt = new("Datatable")
Datatable = function() new("Datatable")


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
    args = as.list(match.call()[-c(1,2)])
    
    #If arg[1] is numeric then restrict the result to that set of rows, regardless of the other arguments
    #If an arg is character then select that column
    #directive = paste(name,":",mode(arg),sep='')
    #if(mode(arg)=='numeric'){
    #  if(name=='i') rows = eval(arg)
    #  else if(name=='j') cols = eval(arg)
    #  else {
    #    directive = paste(directive," const(",arg,")",sep='')
    #  }
    #}
    #else if(mode(arg)=='call'){
    #  func = arg[[1]]
    #  directive = paste(directive,":",func,sep='')
    #}
    
    rows = -1
    cols = -1
    directives = NULL
    
    # Intialise a list of names that refer to column in the Datatable
    # or functions in the dataset. Other names will be searched for in the 
    # R parent frame
    datatable_names = dataquery_directives_
    for(name in x$names()){
      datatable_names[[name]] = Column(name)
    }
    
    for(index in 1:length(args)){
      arg = args[[index]]
      arg = substitute(arg)
      
      # Evaluate each argument with the parent frame as a "fallback" for symbols not in the database
      # See subset.data.frame for an example of this
      directive = tryCatch(eval(arg,datatable_names,parent.frame()),error=function(error) error)
      if(inherits(directive,'error')){
        stop(paste("in query :",directive$message,sep=''),call.=FALSE)
      }
      
      directives = c(directives,directive)
    }
    
    q = do.call(Dataquery,directives)
    q$sql()
  }
)

# S3 methods
#methods(class='data.frame')
#	subset
#	merge
#	summary

#' Get the dimensions of a Datatable
#'
#' @method dim Datatable
#' @export
# Implementing dim also provides for nrow and ncol
dim.Datatable = function(x) x$dimensions()

#' Get the first n rows of a Datatable
#'
#' @method head Datatable
#' @export
head.Datatable = function(x,n=10) as.data.frame(x$head(n))

#' Get the last n rows of a Datatable
#'
#' @method tail Datatable
#' @export
tail.Datatable = function(x,n=10) as.data.frame(x$tail(n))

#' Convert a Datatable to a data.frame
#'
#' @method as.data.frame Datatable
#' @export
as.data.frame.Datatable = function(x) x$dataframe()

#######################################################################

class_('Expression')

# Create an expression element
expr_ <- function(class,...){
  create_('Expression',paste('Expression_',class,sep=''),...)
}

wrap_ <- function(object){
  if(inherits(object,'Expression'))  return(object)
  # Convert fundamental types
  if(inherits(object,'logical')) return(expr_('Logical',object))
  if(inherits(object,'integer')) return(expr_('Integer',object))
  if(inherits(object,'numeric')) return(expr_('Numeric',object))
  if(inherits(object,'character')) return(expr_('String',object))
  
  # If got to here then raise an error
  stop(paste("Object of class",paste(class(object),collapse=","),"is not wrappable"),call.=FALSE)
}
wrap_all_ <- function(...){
  #Create a list from arguments
  args <- list(...)
  #Create a vector of Dataquery expressions by wrapping each argument
  expressions <- vector()
  n <- length(args)
  if(n>0){
    for(index in 1:n) expressions <- c(expressions,wrap_(args[[index]])@pointer)
  }
  expressions
}

#' Create a data query constant
#' 
#' This function is exported mainly for use in testing data queries
#' @export
Constant <- function(object) wrap_(object)

#' Create a Datatable column identifier
#' 
#' This function is exported mainly for use in testing data queries
#' @export
Column <- function(name) expr_('Column',name)

###########################################################################
# Operators
#
# Use getMethod() to obtain the correct function argument names e.g.
#   getMethod("==")

# Unary operator
unop_ <- function(class,expr){
  expr_(class,wrap_(expr)@pointer)
}
setMethod("+",signature(e1='Expression',e2='missing'),function(e1,e2) unop_('Positive',e1))
setMethod("-",signature(e1='Expression',e2='missing'),function(e1,e2) unop_('Negative',e1))
setMethod("!",signature(x='Expression'),function(x) unop_('Not',x))

# Binary operators
# For each binary operator, set a method with an Expression on both sides          
binop_ <- function(class,left,right){
  expr_(class,wrap_(left)@pointer,wrap_(right)@pointer)
}
binop_methods_ <- function(op,name){
  setMethod(op,signature(e1='Expression'),function(e1,e2) binop_(name,e1,e2))
  setMethod(op,signature(e2='Expression'),function(e1,e2) binop_(name,e1,e2))
  setMethod(op,signature(e1='Expression',e2='Expression'),function(e1,e2) binop_(name,e1,e2))
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
Call = function(name,...) {
  expr_('Call',name,wrap_all_(...))
}

#' @export
Distinct <- function(expr) expr_('Distinct')

#' @export
All <- function(expr) expr_('All')

#' @export
Where <- function(expr) expr_('Where',wrap_(expr)@pointer)

#' @export
By <- function(expr) expr_('By',wrap_(expr)@pointer)

func_ <- function(name){
  function(...) Call(name,...)
}
dataquery_directives_ <- list(
  distinct = Distinct,
  all = All,
  where = Where,
  by = By,
  
  abs = func_('abs'),
  min = func_('min'),
  max = func_('max'),
  
  sum = func_('sum')
  #' @todo etc
)

#' The Dataquery class
#'
#' @name Dataquery-class
#' @rdname Dataquery-class
#' @exportClass Dataquery
class_('Dataquery')

#' Create a Dataquery
#'
#' @export
Dataquery = function(...) {
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

#########################################

#' The Stencil class
#'
#' @name Stencil-class
#' @rdname Stencil-class
#' @exportClass Stencil
class_('Stencil')

#' Create a stencil
#'
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
Stencil <- function(content) {
  stencil <- new("Stencil")
  if(!missing(content)){
    stencil$load(content)
  }
  return(stencil)
}

#' Render a stencil object or a stencil string 
#'
#' @export
#'
render <- function(stencil,context){
  if(!('Stencil' %in% class(stencil))){
    stencil <- Stencil(stencil)
  }
  if(missing(context)) context <- Context(parent.frame())
  else {
    if(!('Context' %in% class(context))) context <- Context(context)
  }
  stencil$render(context)
  return(stencil$dump())
}

#' Create a stencil rendering context
#' 
#' Stencils are rendered within a context. 
#'
#' @export
Context <- function(envir){
  
  self <- new.env()
  class(self) <- "Context"
  
  if(missing(envir)) envir <- new.env(parent=baseenv())
  else if(is.list(envir)) envir <- list2env(envir,parent=baseenv())
  self$stack <- list(envir)
  
  ##################################
  
  self$push <- function(item){
    self$stack[[length(self$stack)+1]] <- item
  }
  
  self$pop  <- function() {
    self$stack[[length(self$stack)]] <- NULL
  }
  
  self$top  <- function() {
    return(self$stack[[length(self$stack)]])
  }
  
  ##################################
  
  self$get <- function(expression) {
    return(eval(parse(text=expression),envir=self$top()))
  }
  
  self$set <- function(name,expression){
    env = self$top()
    value <- eval(parse(text=expression),envir=env)
    assign(name,value,envir=env)
  }
  
  ##################################
  # "script" elements
  # 
  # Executes the script
  
  self$script <- function(code){
    eval(parse(text=code),envir=self$top())
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
    return(env)
  }
  
  self$exit <- function(){
    self$pop()
  }
  
  ##################################
  # "each" elements
  #
  # Call begin('item','items') at start of an "each" element
  # Call step() at end of each element
  
  self$begin <- function(item,items){
    # Enter a new anonymous block that forms the namespace for the loop
    loop = self$enter()
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
    loop = self$top()
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

#########################################

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
#' Default iterate method
#' @export
iterate.default <- function(container){
  return(DefaultIterator(container))
}

# Other iterate methods will be added when
# other iterator classes are written (e.g. DataframeIterator, MatrixIterator)

# A default iterator for vectors and lists
DefaultIterator <- function(container){
  self <- new.env()
  class(self) <- "DefaultIterator"
  
  self$container = container
  self$index = 0
  self$size = length(self$container)
  
  self$more <- function(){
    return(self$index<self$size)
  }
  
  self$step <- function(){
    self$index = self$index +1
    return(self$container[[self$index]])
  }
  
  self
}

#########################################

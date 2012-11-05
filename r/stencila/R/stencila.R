#' Stencila R package
#'
#' @docType package
#' @name stencila
#' @aliases stencila stencila-package
#' @author Nokome Bentley <nokome.bentley@@stenci.la>
#' @useDynLib stencila_
NULL

# A convienience function for calling C++ functions
# in the extension module stencila.so
call_ = function(symbol,...){
	.Call(symbol,...,package='stencila')
}

# A convienience function for converting an object
object_ = function(object){
  if(typeof(object)=='externalptr'){
    tag = call_('tag',object)
    object = new(tag,created=TRUE,pointer=object)
    return(object)
  }
  return(object)
}

# A convienience function for creating an S4 class
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
      .Object@pointer = call_(paste(class_name,'new',sep='_'),...)
      .Object@created = TRUE
    } else {
      .Object@pointer = pointer
      .Object@created = TRUE
    }
    return(.Object)
  },list(class_name=class_name))))
  
  setMethod('$',class_name,eval(substitute(function(x,name){
    function(...) {
      result = call_(paste(class_name,name,sep='_'),x@pointer,...)
      return(object_(result))
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
  call_('stencila_version')
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
Dataset = function() new("Dataset")

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
		
		#Iterate through args to determine type and what to do next. This needs to 
		#be done with substitute with parent frame
		
		#If arg[1] is numeric then restrict the result to that set of rows, regardless of the other arguments
		#If an arg is character then select that column
		rows = -1
		cols = -1
		directives = NULL
		
		names = names(args)
		for(index in 1:length(args)){
			name = names[[index]]
			arg = args[[name]]
			arg = substitute(arg)
			directive = paste(name,":",mode(arg),sep='')
			if(mode(arg)=='numeric'){
				if(name=='i') rows = eval(arg)
				else if(name=='j') cols = eval(arg)
				else {
					directive = paste(directive," const(",arg,")",sep='')
				}
			}
			else if(mode(arg)=='call'){
				func = arg[[1]]
				directive = paste(directive,":",func,sep='')
			}
			
			#See subset.data.frame for how we can evaluate each argument
			#with the parent frame as a "fallback" for symbols not in the database
      dataset_names = list(
        #Needs to initialise for each of the names in the database
        year = new("Column","year"),
        region = new("Column","region"),
        sales = new("Column","sales"),
        
        by = function(.) paste("by(",.,")",sep=""),
        sum = function(.) paste("sum(",.,")",sep=""),
        where = function(.) paste("where(",.,")",sep="")
      )
			directive = eval(arg,dataset_names,parent.frame())
      
			directives = c(directives,directive)
		}
		

		
		list(args=args,rows=rows,cols=cols,directives=directives)
	}
)

# S3 methods
#methods(class='data.frame')
#	head & tail
#	subset
#	merge
#	summary

#' Get the dimensions of a Datatable
#'
#' @method dim Datatable
dim.Datatable = function(x) x$dimensions()

#' Convert a Datatable to a data.frame
#'
#' @method as.data.frame Datatable
as.data.frame.Datatable = function(x) x$dataframe()

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

#' @include shortcuts.R
NULL

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

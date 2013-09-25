# Convienience functions used internally in the Stencila package and
# not exported

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

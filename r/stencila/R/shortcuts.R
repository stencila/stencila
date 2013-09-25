# Convienience functions used internally in the Stencila package and not exported
#
# This file has to be 'sourced' before other files in the package.
# Whilst the roxygen2 directive @include shortcuts.R is meant to do that
# it seemed not to work as intended when roxygenising the package
# So this file is specified first in the Collate section of the DESCRIPTION.template (which roxygen checks first)

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
            symbol <- paste(class_name,name,sep='_')
            # Look for a R version of symbol and call it, otherwise
            # try to get it from C++ symbols 
            if(exists(symbol)){
                result <- get(symbol)(x,...)
            } else {
                result <- tryCatch(
                    call_(symbol,x@pointer,...),
                    error = function(error) error
                )
                if(inherits(result,'error')){
                    if(result$message==paste('C symbol name "',symbol,'" not in load table',sep='')){
                        stop(paste('Class ',class_name,' does not have a method named "',name,'"',sep=''),call.=FALSE)
                    } else {
                        stop(result$message,call.=FALSE)
                    }
                }
            }
            #If the return is NULL (in C++ nil) then return self
            #so method chaining can be used...
            if(is.null(result)) return(invisible(x))
            #...otherwise return the object after wrapping it
            else return(object_(result))
            #We could just get C++ functions to return self
            #and wrap the return regardless of type but that creates a new object
            #and would seem to be wateful (and perhaps dangerous?)
        }
    },list(class_name=class_name))))
    
    NULL
}

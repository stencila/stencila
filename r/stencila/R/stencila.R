#' Stencila R package
#'
#' @docType package
#' @name stencila
#' @aliases stencila stencila-package
#' @author Nokome Bentley <nokome.bentley@@stenci.la>
NULL

# This file has to be 'sourced' before other files in the package.
# Whilst the roxygen2 directive @include stencila.R is meant to do that
# it seemed not to work as intended when roxygenising the package
# So this file is specified first in the Collate section of the DESCRIPTION.template (which roxygen checks first)

# The name of the stencila DLL.
# Used internally to help resolution of C++ functions (see shortcuts.R)
dll_name <- NULL

###########################################################################
# Package version number
###########################################################################

#' Get the version of the Stencila R package
#'
#' @examples
#'   stencila:::version()
version <- function(){
	utils::packageVersion("stencila")
}

###########################################################################
# Package startup and shutdown hooks
###########################################################################

# Package startup hook
# See ?.onLoad
.onLoad <- function(libname, pkgname){
	# Call C++ startup function
	.Call('Stencila_startup')
	# Initialise `dll_name`
	dll_name <<- paste('stencila',version(),sep='_')
}

# Package shutdown hook
# See ?.onUnLoad
.onUnload <- function(libpath){
	.Call('Stencila_shutdown')
}

###########################################################################
# Helper functions used internally in the Stencila package and not exported
###########################################################################

# Call a C++ function in the Stencila R extension module
call_ <- function(symbol,...){
    .Call(symbol,...,PACKAGE=dll_name)
}

# Convert an externalptr object into an instance of a Stencila R class
wrap_ <- function(object){
    if(typeof(object)=='externalptr'){
        class <- call_("Stencila_class",object)
        object <- new(class,created=TRUE,pointer=object)
        return(object)
    }
    return(object)
}

# Call a method of a Stencila R class, using either a 
# R function if available, or a C++ function
# if not.
method_ <- function(instance,name,class,bases,...){
    result <- NA
    for(prefix in c(class,bases)){
        # Create a symbol of form `<class>_<name>`
        symbol <- paste(prefix,name,sep='_')
        # Look for a R version of symbol, otherwise
        # try to get it from C++ symbols 
        if(exists(symbol)){
            # Found R function, call it with args
            result <- get(symbol)(instance,...)
            break
        }
        else if(is.loaded(symbol,PACKAGE=dll_name)){
            # Found C++ function, call it with args
            result <- call_(symbol,instance@pointer,...)
            break
        }
    }
    # If the return is NULL then return self
    # so method chaining can be used...
    if(is.null(result)) return(invisible(instance))
    # If result is still NA then the method must not have been found
    else if(is.na(result)){
        stop(paste("Class '",class,"' has no method called '",name,"'",sep=''))
    }
    # ...otherwise return the instance after wrapping it
    else return(wrap_(result))
}

# Creates a function `<class>_<name>` which forwards on to
# the C++ functions `<class>_<name>_get` or `<class>_<name>_set`
# depending on whether value argument is supplied
attr_ <- function(class,name){
    assign(paste(class,name,sep='_'),function(instance,value){
        if(missing(value)) call_(paste(class,name,'get',sep='_'),instance@pointer)
        else call_(paste(class,name,'set',sep='_'),instance@pointer,value)
    },pos=parent.frame())
}

# A convienience function for defining a Stencila R class
class_ <- function(class,bases){
    # Define the class and its bases
    setClass(
        class,
        representation=representation(
            created = 'logical',
            pointer = 'externalptr'
        ),
        prototype=prototype(
            created = FALSE
        )     
    )
    # Set its initialiser
    setMethod('initialize',class,function(.Object,created=FALSE,pointer=NULL,...){
        if(!created){
            .Object@pointer = call_(paste(class,'new',sep='_'))
            .Object@created = TRUE
        } else {
            .Object@pointer = pointer
            .Object@created = TRUE
        }
        return(.Object)
    })
    # Define its getter
    setMethod('$',class,function(x,name){
        function(...) method_(x,name,class,bases,...)
    })
    
    NULL
}

# Create an instance of a Stencila R class
# from a function other than "<class>_new"
create_ <- function(class,func,...){
    new(class,created=TRUE,pointer=call_(func,...))
}

#' Stencila R package
#'
#' @docType package
#' @name stencila
#' @aliases stencila stencila-package
#' @author Nokome Bentley <nokome.bentley@@stenci.la>
#'
#' @import grDevices
#' @import methods
#' @import stats
#' @import utils
#'
#' @useDynLib extension
NULL

# This file has to be 'sourced' before other files in the package.
# Whilst the roxygen2 directive @include stencila.R is meant to do that
# it seemed not to work as intended when roxygenising the package
# So this file is specified first in the Collate section of the DESCRIPTION.template (which roxygen checks first)

###########################################################################
# Package version number and other global attributes
###########################################################################

#' Get the version of the Stencila R package
#'
#' @param Should the version be obtained from the DLL? Default FALSE
#'
#' @examples
#'   stencila:::version()
version <- function(dll=FALSE){
	if(dll) .Call('Stencila_version',PACKAGE='stencila')
	else as.character(utils::packageVersion("stencila"))
}

#' Get the commit hash of the Stencila R package
#'
#' @examples
#'   stencila:::commit()
commit <- function(){
	.Call('Stencila_commit',PACKAGE='stencila')
}

#' Get the Stencila stores
#'
#' @examples
#'   stencila:::stores()
stores <- function(){
	.Call('Stencila_stores',PACKAGE='stencila')
}

#' Start the built in server
#'
#' @export
#' @examples
#'   stencila:::serve()
serve <- function(){
	.Call('Stencila_serve',PACKAGE='stencila')
}

###########################################################################
# Package startup and shutdown hooks
###########################################################################

.onLoad <- function(libname, pkgname){
	# Attempt to load DLL
	loaded <- load_dll()
	# On Linux, it seems OK to attempt to install the DLL within this function. 
	# But on windows, that causes building of the package (`R CMD INSTALL ---build`) to fail
	# So, only do it for linux...
	if(!loaded && grepl('linux',R.version$platform)){
		# Since R does a test install first, don't attempt to download the DLL
		# when doing this (otherwise the download is done twice
		# i.e. this will only work for the package which already has the DLL bundled in the `bin` dir
		install_dll(download=FALSE,load=TRUE)
	}
}

.onAttach <- function(libname, pkgname){
	# Call C++ startup function
	# Protect from failiure so this function, which is called during packages installation,
	# does not fail
	result <- tryCatch(.Call('Stencila_startup',PACKAGE='stencila'),error=identity)
	if(inherits(result,'simpleError')){
		warning("Stencila DLL does not appear to be installed. Please run `stencila:::install_dll()`.")
	}
}

.onDetach <- function(lib){
	# Call C++ shutdown function
	.Call('Stencila_shutdown',PACKAGE='stencila')
}

.onUnload <- function(libpath){
	# Unload the DLL
	tryCatch(library.dynam.unload('stencila',system.file(package='stencila')),silent=TRUE)
}

###########################################################################
# C++ extension base class
###########################################################################

#' The Extension class
#'
#' @name Extension
#' @export
Extension <- function() {
    new('Extension')
}
setRefClass(
	'Extension',
	fields = list(
		# A 'private' field which holds the pointer to the C++-side
		# representation of this instance
		.pointer = 'externalptr'
	),
	methods = list(
		initialize = function(pointer=NULL,...){
			callSuper(...)
			if(is.null(pointer)) pointer <- call_(paste0(class(.self)[1],'_new'))
			.pointer <<- pointer
		}
	)
)

# Convenience functions for working with Stencila R `Extension`es.
# To avoid polluting the method list of classes these are not declared as 
# methods

# Call a C++ function in the Stencila R extension module
call_ <- function(symbol,...){
    .Call(symbol,...,PACKAGE='stencila')
}
# Call a method and wrap the result appropriately
method_ <- function(instance,symbol,...){
    # Do the call
    result <- call_(symbol,instance$.pointer,...)
    # If the return is NULL then return instance
    # so method chaining can be used...
    if(is.null(result)) return(invisible(instance))
    # Otherwise return the result after wrapping it
    else {
        # Convert an externalptr object into an instance of a Stencila R class
        if(typeof(result)=='externalptr'){
            class <- call_('Stencila_class',result)
            result <- new(class,pointer=result)
            return(result)
        }
        return(result)
    }
}
# Call a getter function; raise an error if an attempt
# is made to set the field
get_ <- function(instance,symbol,value){
    if(missing(value)) method_(instance,symbol)
    else stop('Read only field')
}
# Call setter function
set_ <- function(instance,symbol,value){
    method_(instance,symbol,value)
}
# Call getter or setter function as appropriate
get_set_ <- function(instance,get_symbol,set_symbol,value){
    if(missing(value)) method_(instance,get_symbol)
    else method_(instance,set_symbol,value)
}
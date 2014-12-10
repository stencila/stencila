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
# Used internally to help resolution of C++ functions
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

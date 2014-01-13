#' Stencila R package
#'
#' @docType package
#' @name stencila
#' @aliases stencila stencila-package
#' @author Nokome Bentley <nokome.bentley@@stenci.la>
NULL

# Package startup hook
# See ?.onLoad
.onLoad <- function(libname, pkgname){
	.Call('Stencila_startup')
}

# Package shutdown hook
# See ?.onUnLoad
.onUnload <- function(libpath){
	.Call('Stencila_shutdown')
}

#' Get the version of the Stencila R package
#'
#' @examples
#'   stencila:::version()
version <- function(){
	.Call('Stencila_version')
}

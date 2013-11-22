#' Stencila R package
#'
#' @docType package
#' @name stencila
#' @aliases stencila stencila-package
#' @author Nokome Bentley <nokome.bentley@@stenci.la>
#' @import utils
#' @include shortcuts.R
NULL

# Package startup hook
# See ?.onLoad
.onLoad <- function(libname, pkgname){
  call_('Stencila_startup')
}

# Package shutdown hook
# See ?.onUnLoad
.onUnload <- function(libpath){
}

#' Get the version of the Stencila R package
#'
#' @examples
#'   stencila:::version()
version <- function(){
  call_('Stencila_version')
}

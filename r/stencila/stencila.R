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
	as.character(utils::packageVersion("stencila"))
}

###########################################################################
# Installation of packaged shared libraries (.so, .dll) and command line 
# scripts
###########################################################################

#' Install `stencila-r` on the sytem path
#'
#' On Linux this function creates a symlink to `stencila-r` in `/usr/local/bin`. 
#' Use R as a superuser (e.g. with `sudo`) to run this function e.g :
#'
#'   \code{ sudo Rscript -e 'require(stencila); stencila:::install_cli()' }
#'
install_cli <- function(){
	src <- file.path(system.file(package='stencila'),'bin','stencila-r')
	dest <- file.path('/usr/local/bin','stencila-r')
	suppressWarnings(file.remove(dest))
	ok <- file.symlink(src,dest)
	if(ok) cat('Stencila CLI(command line interface) for R installed to:',dest,'\n')
}

#' Install `stencila.min.js` so it can be served from the localhost
install_js <- function(){
	src <- file.path(system.file(package='stencila'),'bin','stencila.min.js')
	dest <- file.path(.Call('Stencila_home'),'stencila.min.js')
	ok <- file.copy(src,dest,overwrite=T)
	if(ok) cat('Stencila Javascript installed to:',dest,'\n')
}

#' Install extra Stencila scripts
#'
#' On Linux, use R as a superuser (e.g. with `sudo`) to run this function e.g :
#'
#'   \code{ sudo Rscript -e 'require(stencila); stencila:::install()' }
#'
install <- function(){
	install_cli()
	install_js()
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

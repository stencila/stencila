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

r_platform <- function(){
	R.version$platform
}
r_version <- function(){
	paste(R.version$major,strsplit(R.version$minor,'\\.')[[1]][1],sep='.')
}
r_dll_ext <- function(){
	.Platform$dynlib.ext
}

# The name of the Stencila DLL.
# Used internally to help resolution of C++ functions
dll_name <- NULL

dll_file <- function(){
	paste0('stencila_',version(),r_dll_ext())
}

dll_path <- function(){
	file.path('bin',paste0(dll_file(),'.zip'))
}

dll_url <- function(){
	paste0("http://get.stenci.la/r/bin/",r_platform(),"/",r_version(),"/",dll_file(),'.zip')
}

# Flag for if Stencila DLL is loaded or not.
dll_loaded <- FALSE

load_dll <- function(){
	if(!dll_loaded){
		result <- tryCatch(library.dynam(dll_name,'stencila',.libPaths()),error=identity)
		dll_loaded <<- !inherits(result,'simpleError')
	}
}

install_dll <- function(){
	message("Installing Stencila DLL")
	# See if it is available locally in the "dll" folder
	zip_path <- file.path(system.file(package='stencila'),dll_path())
	message(" - finding ",zip_path)
	if(!file.exists(zip_path)){
	  # If it is not then download it to a temporary zip file
	  url <- dll_url()
	  zip_path <- tempfile()
	  message(" - downloading ",url)
	  result <- tryCatch(download.file(url,zip_path),error=identity)
	  if(inherits(result,'simpleError')) return(FALSE)
	}
	# Unzip into package `libs`
	libs_dir <- file.path(system.file(package='stencila'),'libs')
	dir.create(libs_dir, recursive = TRUE, showWarnings = FALSE)
	# Unzip the file there
	message(" - unzipping ",zip_path," to ",libs_dir)
	unzip(zip_path, exdir = libs_dir, overwrite = TRUE)
	return(TRUE)
}

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

#' Install extra Stencila scripts
#'
#' On Linux, use R as a superuser (e.g. with `sudo`) to run this function e.g :
#'
#'   \code{ sudo Rscript -e 'require(stencila); stencila:::install()' }
#'
install <- function(){
	install_cli()
}

###########################################################################
# Package startup and shutdown hooks
###########################################################################

.onLoad <- function(libname, pkgname){
	# Initialise `dll_name`
	dll_name <<- paste0('stencila_',version())
	# Attempt to load shared library and install it
	# if it can't be loaded
	load_dll()
	if(!dll_loaded){
		install_dll()
		load_dll()
	}
	# If DLL still not loaded let the user know
	if(!dll_loaded) warning("Stencila DLL could not be loaded. Please ensure you are connected to the internet and try `stencila:::install_dll()` again.")
}

.onAttach <- function(libname, pkgname){
	# Call C++ startup function
	.Call('Stencila_startup',PACKAGE=dll_name)
}

.onDetach <- function(libname, pkgname){
	# Call C++ startup function
	.Call('Stencila_shutdown',PACKAGE=dll_name)
}

.onUnload <- function(libpath){
	library.dynam.unload(dll_name,system.file(package='stencila'))
}

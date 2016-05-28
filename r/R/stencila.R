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
# Package version number and other global attributes
###########################################################################

#' Get the version of the Stencila R package
#'
#' @examples
#'   stencila:::version()
version <- function(package=FALSE){
	if(package) as.character(utils::packageVersion("stencila"))
	else .Call('Stencila_version',PACKAGE='stencila')
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
# Installation of packaged shared libraries (.so, .dll) and command line 
# scripts
###########################################################################

have_dll <- function(){
	'stencila' %in% unlist(lapply(library.dynam(),function(x) x[["name"]]))
}

load_dll <- function(){
	# Check that the DLL is not already loaded
	if(have_dll()){
		invisible(TRUE)
	}
	else {
		# Attempt to load the DLL
		result <- tryCatch(library.dynam('stencila','stencila',.libPaths()),error=identity)
		if(inherits(result,'simpleError')){
			invisible(FALSE)
		} else {
			invisible(TRUE)
		}
	}
}

dll_path <- function(){
	plat <- R.version$platform
	if(grepl('linux',plat)) os <- 'linux' 
	else if(grepl('mingw',plat)) os <- 'win'
	else warning("Stencila DLL is not available for this operating system, sorry.")
	
	arch <- R.version$arch
	r_version <- paste(R.version$major,strsplit(R.version$minor,'\\.')[[1]][1],sep='.')

	# Use the full Stencila version from the VERSION file. Can't use the `version()`
	# function because the DLL is not yet loaded
	stencila_version <- readLines(file.path(system.file(package='stencila'),'bin','VERSION'))[1]

	file.path(os,arch,r_version,paste0('stencila-',stencila_version,'.zip'))
}

#' Download the Stencila dynamically linked libary (DLL)
#'
#'   \code{ sudo Rscript -e 'require(stencila); stencila:::get_dll()' }
#'
get_dll <- function(){
	path <- dll_path()
	# URL to get it from
	url <- paste0('http://get.stenci.la/r/dll/',path)
	# Path to put it to
	zip <- file.path(system.file(package='stencila'),'bin',path)
	# Download it!
	message(" - downloading: ",url)
	result <- tryCatch(download.file(url,zip),error=identity)
	if(inherits(result,'simpleError')){
		warning("Stencila DLL could not be downloaded. Please ensure you are connected to the internet and try again.")
		invisible(FALSE)
	}
	else {
		invisible(TRUE)
	}
}

#' Install the Stencila dynamically linked libary (DLL)
#'
#'   \code{ sudo Rscript -e 'require(stencila); stencila:::install_dll()' }
#'
install_dll <- function(get=TRUE,load=TRUE){
	# Check the DLL is not already loaded (you get a segfault if
	# you try to write over it when it is loaded alrady!)
	if(have_dll()) return(invisible(TRUE))
	message("Installing Stencila DLL")
	# See if it is available locally in the `bin` dir
	zip <- file.path(system.file(package='stencila'),'bin',dll_path())
	message(" - looking for: ",zip)
	# Get the DLL if it is not avialable locally
	if(!file.exists(zip)){
		if(get){
			got <- get_dll()
			if(!got) return(invisible(FALSE))
		}
		else {
			return(invisible(FALSE))
		}
	}
	# Determine where to put the DLL
	libs_dir <- file.path(system.file(package='stencila'),'libs')
	if(grepl('mingw',R.version$platform)){
		if(R.version$arch=='x86_64') libs_dir <- file.path(libs_dir,'x64')
		else libs_dir <- file.path(libs_dir,'i386')
	}
	dir.create(libs_dir, recursive = TRUE, showWarnings = FALSE)
	# Unzip the DLL into the `libs` dir
	message(" - unzipping to: ",libs_dir)
	unzip(zip, exdir = libs_dir, overwrite = TRUE)
	# Now, load it!
	if(load){
		load_dll()
		# ... and call the start up function
		.Call('Stencila_startup',PACKAGE='stencila')
	}

	invisible(TRUE)
}

#' Install `stencila-r` on the system path
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
	install_dll()
	install_cli()
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
		install_dll(get=FALSE,load=TRUE)
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

.onDetach <- function(libname, pkgname){
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
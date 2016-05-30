# Installation and loading of packaged shared libraries (.so, .dll) and 
# command line scripts

# Is the Stencila dynamically linked library already loaded?
have_dll <- function(){
	'stencila' %in% unlist(lapply(library.dynam(),function(x) x[["name"]]))
}

# Load the Stencila dynamically linked library
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

# Get the file name of the zipped Stencila dynamically linked library for this platform
name_dll <- function(){
	stencila_version <- utils::packageVersion("stencila")

	r_version <- paste(R.version$major,strsplit(R.version$minor,'\\.')[[1]][1],sep='.')

	plat <- R.version$platform
	if(grepl('linux',plat)) os <- 'linux' 
	else if(grepl('mingw',plat)) os <- 'win'
	else os <- plat

	arch <- R.version$arch
	
	paste0('stencila-',stencila_version,'-r-',r_version,'-',os,'-',arch,'.zip')
}

#' Get the zipped Stencila dynamically linked library (either locally or by downloading it)
#' and put it in the `bin` subdirectory of the package installation.
#'
#' The environment variable `STENCILA_R_BUILD` can be set to tell the installation to
#' get the zipped DLL from a local build directory
#'
#' @param download Should the DLL be downloaded if it is not available locally? Default TRUE
get_dll <- function(download=TRUE){
	ok <- TRUE
	name <- name_dll()
	dest_dir <- file.path(system.file(package='stencila'),'bin')
	dir.create(dest_dir, recursive = TRUE, showWarnings = FALSE)
	dest <- file.path(dest_dir, name)
	dir <- Sys.getenv('STENCILA_R_BUILD')
	if (nchar(dir)>0) {
		# Available locally so copy over
		src <- file.path(dir, name)
		if(!file.exists(src)) {
			stop("Error file does not exist:",src)
		}
		message(" - copying ", src)
		file.copy(src, dest, overwrite=TRUE)
	} else {
		if(download) {
			# Not available locally so download
			url <- paste0('http://get.stenci.la/r/dll/',name)
			message(" - downloading: ",url)
			result <- tryCatch(download.file(url,dest),error=identity)
			if(inherits(result,'simpleError')){
				warning("Stencila DLL could not be downloaded. Please ensure you are connected to the internet and try again.")
				ok <- FALSE
			}
		} else {
			ok <- FALSE
		}
	}
	invisible(ok)
}

#' Install the Stencila dynamically linked libary (DLL)
#'
#'   \code{ sudo Rscript -e 'require(stencila); stencila:::install_dll()' }
#'
#' @param download Should the DLL be downloaded if it is not available locally? Default TRUE
#' @param download Should the DLL be loaded into the user environment? Default TRUE
install_dll <- function(download=TRUE,load=TRUE){
	# Check the DLL is not already loaded (you get a segfault if
	# you try to write over it when it is loaded alrady!)
	if(have_dll()) return(invisible(TRUE))
	message("Installing Stencila DLL")
	# See if it is available locally in the `bin` dir
	zip <- file.path(system.file(package='stencila'),'bin',name_dll())
	message(" - looking for: ",zip)
	# Get the DLL if it is not avialable locally
	if(!file.exists(zip)){
		got <- get_dll(download=download)
		if(!got) return(invisible(FALSE))
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

#' Install the `stencila-r` command line interface (CLI) on the system path
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

#' Install Stencila dynamically linked library and command line interface (CLI)
#'
#' On Linux, use R as a superuser (e.g. with `sudo`) to run this function e.g :
#'
#'   \code{ sudo Rscript -e 'require(stencila); stencila:::install()' }
#'
install <- function(){
	install_dll()
	install_cli()
}

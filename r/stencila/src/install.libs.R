# This file is for installing pre-compiled binary shared libraries
# See http://cran.r-project.org/doc/manuals/R-exts.html#Package-subdirectories
# 
# The following variables are available when this script is run
#  R_PACKAGE_NAME (the name of the package)
#  R_PACKAGE_SOURCE (the path to the source directory of the package)
#  R_PACKAGE_DIR (the path of the target installation directory of the package)
#  R_ARCH (the arch-dependent part of the path)
#  SHLIB_EXT (the extension of shared objects)
#  WINDOWS (TRUE on Windows, FALSE elsewhere)

message("running 'install.libs.R'")

# Get the R platform and version number which determines which shared library to use
r_platform <- R.version$platform
r_version <- paste(R.version$major,strsplit(R.version$minor,'\\.')[[1]][1],sep='.')
# Get the stencila version which also determines which shared library
stencila_version <- paste(read.dcf("../DESCRIPTION",c("Package","Version")),collapse="_")
# Calculate the name of zipped library file
zipfile <- file.path("lib",r_platform,r_version,paste(stencila_version,SHLIB_EXT,".zip",sep=""))
# See if it is available locally in the "inst" folder
zipfilepath <- file.path("..","inst",zipfile)
message(" - finding ",zipfilepath)
if(!file.exists(zipfilepath)){
  # If it is not then download it to a temporary zipfile
  url <- paste("http://get.stenci.la/r/",zipfile,sep='')
  zipfilepath <- tempfile()
  message(" - downloading ",url)
  download.file(url,zipfilepath)
}
# Calculate where the library needs to do (this code is from the above page)
libsarch <- if (nzchar(R_ARCH)) paste('libs', R_ARCH, sep='') else 'libs'
libsdir <- file.path(R_PACKAGE_DIR, libsarch)
dir.create(libsdir, recursive = TRUE, showWarnings = FALSE)
# Unzip the file there
message(" - unzipping ",zipfilepath)
unzip(zipfilepath, exdir = libsdir, overwrite = TRUE)

message(" - finished")
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
#
# The following code is from the above page. 
# It copied over all the shared library files (for the OS) in this directory

files <- Sys.glob(paste("*", SHLIB_EXT, sep=''))
libarch <- if (nzchar(R_ARCH)) paste('libs', R_ARCH, sep='') else 'libs'
dest <- file.path(R_PACKAGE_DIR, libarch)
dir.create(dest, recursive = TRUE, showWarnings = FALSE)
file.copy(files, dest, overwrite = TRUE)

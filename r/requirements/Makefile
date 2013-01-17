# Makefile for requirements for building the Stencila R package
# Note that these are NOT requirements of USING the Stencila R package.
#
# These requirements could be installed from R using a command like:
#   Rscript -e "install.packages(c('Rcpp','roxygen2','svUnit','XML'),lib='/usr/lib/R/site-library',repos='http://cran.us.r-project.org')"
# However, for windows this will install a binary version of Rcpp, and unless this happens
# to have been built using the same compiler. You are likely to get errors. The R admin manual recommends compiling Rcpp from source
# when used by a package that uses it: http://cran.r-project.org/doc/manuals/R-admin.html#The-MinGW_002dw64-toolchain
#
# Note that the these will install the particular version of these R packages on your system
# If you want a newer version then update the version numbers

RCPP_VERSION = 0.10.2
ROXYGEN2_VERSION = 2.2.2

all : Rcpp roxygen2

.PHONY: Rcpp roxygen2

####################################################

Rcpp_$(RCPP_VERSION).tar.gz:
	wget http://cran.r-project.org/src/contrib/Rcpp_$(RCPP_VERSION).tar.gz
	
Rcpp_$(RCPP_VERSION): Rcpp_$(RCPP_VERSION).tar.gz
	tar -xf Rcpp_$(RCPP_VERSION).tar.gz

Rcpp: Rcpp_$(RCPP_VERSION)
	R CMD INSTALL Rcpp
	
####################################################
	
roxygen2_$(ROXYGEN2_VERSION).tar.gz:
	wget http://cran.r-project.org/src/contrib/roxygen2_$(ROXYGEN2_VERSION).tar.gz
	
roxygen2_$(ROXYGEN2_VERSION): roxygen2_$(ROXYGEN2_VERSION).tar.gz
	tar -xf roxygen2_0.10.2.tar.gz

roxygen2: roxygen2_$(ROXYGEN2_VERSION)
	R CMD INSTALL roxygen2

include ../cpp/Makefile.inc

# R does not like version numbers which are not in like X.Y.Z (or use a - instead of a .)
# So in some place use the latest tag instead of the full git generated version string
TAG := $(shell git describe --abbrev=0)
VERSION :=  $(shell git describe)

.PHONY: compile docs build check

all:  check

# The compile job is for testing compilation without doing the full "R CMD check"
# The files stencila.so and stencila.o are remove to prevent R CMD check from complaining later
RCPP_CXXFLAGS := $(shell Rscript -e "Rcpp:::CxxFlags()")
RCPP_LDFLAGS :=  $(shell Rscript -e "Rcpp:::LdFlags()")
compile:
	sed -i 's!version = .*;$$!version = "$(VERSION)";!' version.hpp
	g++ -I/usr/share/R/include $(RCPP_CXXFLAGS) $(STENCILA_CPP_FLAGS) -fpic -g -c stencila.cpp -o stencila.o;\
	g++ -shared -o stencila.so stencila.o -L/usr/lib/R/lib -lR $(RCPP_LDFLAGS) $(STENCILA_CPP_LIBS);\
	rm stencila.o
	
docs:
	rm -f stencila/man/*.Rd
	Rscript -e "library(roxygen2);roxygenize('stencila');"
	
# Update the description file with the correct version and date
# Using sed:
#	.* = anything, any number of times
#	$ = end of line
# The $ needs to be doubled for escaping make
# ISO 8601 date/time stamp used: http://en.wikipedia.org/wiki/ISO_8601
DATE := $(shell date --utc +%Y-%m-%dT%H:%M:%SZ)
desc:
	sed -i 's!Version: .*$$!Version: $(TAG)!' stencila/DESCRIPTION
	sed -i 's!Date: .*$$!Date: $(DATE)!' stencila/DESCRIPTION
	
# When you run 
#   R CMD build stencila
# it creates a tar.gz file but that file does not include stencila_.so
# So, the folowing line replaces it
#   tar cfz stencila_$(TAG).tar.gz stencila
build: compile docs desc
	mv stencila.so stencila/src/stencila_.so
	tar cfz stencila_$(TAG).tar.gz stencila
	mkdir -p builds
	mv stencila_$(TAG).tar.gz builds
	
check:
	R CMD check builds/stencila_$(TAG).tar.gz
	
install:
	R CMD INSTALL builds/stencila_$(TAG).tar.gz
	
clean:
	rm -rf builds stencila.Rcheck
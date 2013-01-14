include ../cpp/Makefile.common

# R does not like version numbers which are not in like X.Y.Z (or use a - instead of a .)
# So in some place use the latest tag instead of the full git generated version string
TAG := $(shell git describe --abbrev=0)

.PHONY: compile rds docs desc test install

all:  clean test

# Requires everything that Stencila C++ library requires and the Stencila C++ library itself
requirements:
	make -C ../cpp/requirements all
	make -C ../cpp/lib all

# Compilation

STENCILA_R_CPPS := $(wildcard *.cpp)
STENCILA_R_OS := $(patsubst %.cpp,%.o,$(STENCILA_R_CPPS))
RCPP_CXXFLAGS := $(shell Rscript -e "Rcpp:::CxxFlags()")
RCPP_LDFLAGS :=  $(shell Rscript -e "Rcpp:::LdFlags()")

%.o : %.cpp
	g++ -c -O2 -fPIC $(RCPP_CXXFLAGS) $(STENCILA_CPP_FLAGS) -I/usr/share/R/include $(STENCILA_CPP_INCLUDE_DIRS) -o$@ $<

compile: version.hpp requirements $(STENCILA_R_OS)
	g++ -shared $(STENCILA_R_OS) -ostencila.so -L/usr/lib/R/lib -lR $(RCPP_LDFLAGS) $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS)

# Update the description file with the correct version and date
# The DESCRIPTION.template file is copied so that there is not a
# change in the repository just when a build is done
# Using sed:
#	.* = anything, any number of times
#	$ = end of line
# The $ needs to be doubled for escaping make
# ISO 8601 date/time stamp used: http://en.wikipedia.org/wiki/ISO_8601
DATE := $(shell date --utc +%Y-%m-%dT%H:%M:%SZ)
desc:
	cp DESCRIPTION.template stencila/DESCRIPTION
	sed -i 's!Version: .*$$!Version: $(TAG)!' stencila/DESCRIPTION
	sed -i 's!Date: .*$$!Date: $(DATE)!' stencila/DESCRIPTION

#Generate Rd files using roxygen
# At some point Rscript -e "library(roxygen2);roxygenize('stencila');" stopped
# working so echoing into R is used
# See http://stackoverflow.com/questions/8964515/cannot-call-roxygenize-function-from-rscript-batch-file
rds: desc
	rm -f stencila/man/*.Rd
	echo "library(roxygen2);roxygenize('stencila')" | R --vanilla
    
# Create the package
# When you run 
#   R CMD build stencila
# it creates a tar.gz file but that file does not include stencila_.so
# So, the folowing line replaces R CMD build:
#   tar cfz stencila_$(TAG).tar.gz stencila
package: rds
	rm -f stencila/src/*.o stencila/src/stencila.so stencila/src/symbols.rds #Remove any old build artifacts
	cp stencila.so stencila/src/stencila_.so #Copy in the compiles extension
	tar cfz stencila_$(TAG).tar.gz stencila #Tarball it up
	mkdir -p repo/src/contrib #Make an R repository source package directory
	mv stencila_$(TAG).tar.gz repo/src/contrib #Move it there
	echo "tools::write_PACKAGES('repo/src/contrib',type='source')" | R --vanilla #Generate the PACKAGE file for the repo

# Generate Stencila custom HTML documentation
docs: rds
	$(MAKE) -C docs

build: compile package

# Check the package by running R CMD check
r_check: build
	R CMD check repo/src/contrib/stencila_$(TAG).tar.gz

# Test the package by running unit tests
# Install package in a testing directory and run unit tests from there
# This is better than installing package in the users R library location
unit_tests: build
	mkdir -p testing
	R CMD INSTALL -l testing repo/src/contrib/stencila_$(TAG).tar.gz
	echo "setwd('testing'); library(stencila,lib.loc='.'); source('stencila/unitTests/do.svUnit.R')" | R --vanilla

test: r_check unit_tests

install:
	R CMD INSTALL repo/src/contrib/stencila_$(TAG).tar.gz

clean:
	rm -rf *.o *.so stencila.Rcheck testing

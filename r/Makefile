include ../cpp/Makefile.common

# R does not like version numbers which are not in like X.Y.Z (or use a - instead of a .)
# So in some place use the latest tag instead of the full git generated version string
TAG := $(shell git describe --abbrev=0)

.PHONY: compile docs desc test install

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
docs: desc
	rm -f stencila/man/*.Rd
	echo "library(roxygen2);roxygenize('stencila')" | R --vanilla
    
# Create the package
# When you run 
#   R CMD build stencila
# it creates a tar.gz file but that file does not include stencila_.so
# So, the folowing line replaces R CMD build:
#   tar cfz stencila_$(TAG).tar.gz stencila
package: docs
	cp stencila.so stencila/src/stencila_.so
	tar cfz stencila_$(TAG).tar.gz stencila
	mkdir -p builds
	mv stencila_$(TAG).tar.gz builds
	ln -sf stencila_$(TAG).tar.gz builds/stencila_latest.tar.gz 

build: compile package

check:
	R CMD check builds/stencila_$(TAG).tar.gz

# Test the package by running unit tests
# Install package in a testing directory and run unit tests from there
# This is better than installing package in the users R library location
test: build
	mkdir -p testing
	R CMD INSTALL -l testing builds/stencila_latest.tar.gz
	echo "setwd('testing'); library(stencila,lib.loc='.'); source('stencila/unitTests/do.svUnit.R')" | R --vanilla

install:
	R CMD INSTALL builds/stencila_$(TAG).tar.gz

clean:
	rm -rf *.o *.so stencila.Rcheck testing

all: test repo

.PHONY:

include $(dir $(lastword $(MAKEFILE_LIST)))/../cpp/variables.make

# Requires everything that Stencila C++ library requires and the Stencila C++ library itself
requires:
	$(MAKE) -C ../cpp/requires all
	$(MAKE) -C ../cpp version.hpp

# Define R and Rcpp compile flags
R_CPPFLAGS := $(shell R CMD config --cppflags)
R_LDFLAGS := $(shell R CMD config --ldflags)
RCPP_CXXFLAGS := $(shell Rscript -e "Rcpp:::CxxFlags()")
RCPP_LDFLAGS :=  $(shell Rscript -e "Rcpp:::LdFlags()")

# Under MSYS it is necessary to change "c:" in paths to "/c/"
# Obviously this will fail if R is not install on C drive
# Compilation will also fail later if there are any space in the R home path
ifeq ($(STENCILA_PLATFORM),msys)
R_CPPFLAGS := $(shell echo "$(R_CPPFLAGS)" | sed "s/c:/\/c/g")
R_LDFLAGS := $(shell echo "$(R_LDFLAGS)" | sed "s/c:/\/c/g")
RCPP_CXXFLAGS := $(shell echo "$(RCPP_CXXFLAGS)" | sed "s/c:/\/c/g")
RCPP_LDFLAGS := $(shell echo "$(RCPP_LDFLAGS)" | sed "s/c:/\/c/g")
endif

# Define R platform and version strings including the R major version
# Note in the below the double $ is to escape make's treatment of $
# and the \$ is to escape the shell's treatment of $
STENCILA_R_PLATFORM := $(shell Rscript -e "cat(R.version\$$platform)" )
# Version number excludes any patch number
STENCILA_R_VERSION := $(shell Rscript -e "cat(R.version\$$major,strsplit(R.version\$$minor,'\\\\.')[[1]][1],sep='.')" )

# Define other platform specific variables...
ifeq ($(STENCILA_PLATFORM),linux)
STENCILA_R_PACKAGE := stencila_$(STENCILA_VERSION).tar.gz
STENCILA_R_REPO_DIR := repo/src/contrib
STENCILA_R_REPO_TYPE := source
STENCILA_R_LIB_EXT := so
endif
ifeq ($(STENCILA_PLATFORM),msys)
STENCILA_R_PACKAGE := stencila_$(STENCILA_VERSION).zip
STENCILA_R_REPO_DIR := repo/bin/windows/contrib/$(STENCILA_R_VERSION)
STENCILA_R_REPO_TYPE := win.binary
STENCILA_R_LIB_EXT := dll
endif
# Define where the shared library gets put
STENCILA_R_LIB_DIR := lib/$(STENCILA_R_PLATFORM)/$(STENCILA_R_VERSION)
STENCILA_R_LIB_NAME := stencila_$(STENCILA_VERSION)
STENCILA_R_LIB_FILE := $(STENCILA_R_LIB_NAME).$(STENCILA_R_LIB_EXT)
STENCILA_R_LIB_ZIPFILE := $(STENCILA_R_LIB_FILE).zip


# Define lists of C++ files and corresponding .o files
STENCILA_R_CPPS := $(wildcard stencila/inst/src/*.cpp)
STENCILA_R_OS := $(patsubst %.cpp,%.o,$(STENCILA_R_CPPS))

%.o : %.cpp
	$(CXX) -c -O2 -fPIC $(R_CPPFLAGS) $(RCPP_CXXFLAGS) $(STENCILA_CPP_FLAGS) $(STENCILA_CPP_INCLUDE_DIRS) -o$@ $<

build:
	mkdir build

build/$(STENCILA_R_LIB_FILE): requires build $(STENCILA_R_OS)
	$(CXX) -shared $(STENCILA_R_OS) -o build/$(STENCILA_R_LIB_FILE) $(R_LDFLAGS) $(RCPP_LDFLAGS) $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS)

build/$(STENCILA_R_LIB_ZIPFILE): build/$(STENCILA_R_LIB_FILE)
	rm -f build/$(STENCILA_R_LIB_ZIPFILE)
	zip -j build/$(STENCILA_R_LIB_ZIPFILE) build/$(STENCILA_R_LIB_FILE)

# Update the description file with the correct version and date
# The DESCRIPTION.template file is copied so that there is not a
# change in the repository just when a build is done
# Using sed:
#	.* = anything, any number of times
#	$ = end of line
# The $ needs to be doubled for escaping make
# ISO 8601 date/time stamp used: http://en.wikipedia.org/wiki/ISO_8601
DATE := $(shell date --utc +%Y-%m-%dT%H:%M:%SZ)
description:
	cp DESCRIPTION.template stencila/DESCRIPTION
	sed -i 's!Version: .*$$!Version: $(STENCILA_VERSION)!' stencila/DESCRIPTION
	sed -i 's!Date: .*$$!Date: $(DATE)!' stencila/DESCRIPTION

# Generate Rd files using roxygen
rds: description
	rm -f stencila/man/*.Rd
	Rscript -e "library(roxygen2);roxygenize('stencila');"

# Add useDynLib to the NAMESPACE file
namespace:
	echo "useDynLib($(STENCILA_R_LIB_NAME))" >> stencila/NAMESPACE
    
# Create the package
$(STENCILA_R_PACKAGE): build/$(STENCILA_R_LIB_ZIPFILE) rds namespace
	# Get rid of any unwanted files from package
	rm -rf stencila/R/*.Rhistory stencila/inst/src/*.o stencila/src/*.o stencila/src/*.so stencila/src/*.dll
	# Copy the library into inst so it can be installed locally for testing
	mkdir -p stencila/inst/$(STENCILA_R_LIB_DIR)
	cp build/$(STENCILA_R_LIB_ZIPFILE) stencila/inst/$(STENCILA_R_LIB_DIR)
ifeq ($(STENCILA_PLATFORM),linux)
	R CMD build stencila
endif
ifeq ($(STENCILA_PLATFORM),msys)
	R CMD INSTALL --build stencila #Build it as per usual for windows
endif

package: $(STENCILA_R_PACKAGE)

# Check the package by running R CMD check
# CMD check requires package task for:
# 	- DESCRIPTION file
# 	- Rd files
# 	- compiled library file
r_check: package
	R CMD check stencila

# Test the package by running unit tests
# Install package in a testing directory and run unit tests from there
# This is better than installing package in the users R library location
unit_tests: testing $(STENCILA_R_PACKAGE)
	R CMD INSTALL -l testing $(STENCILA_R_PACKAGE)
	cd testing; Rscript -e "library(stencila,lib.loc='.'); setwd('stencila/unitTests/'); source('do-svUnit.R')"

testing:
	mkdir testing

test: r_check unit_tests

# Deposit package into local repository
# See http://cran.r-project.org/doc/manuals/R-admin.html#Setting-up-a-package-repository
repo: $(STENCILA_R_PACKAGE)
	# Make R package repository sub directory
	mkdir -p  $(STENCILA_R_REPO_DIR)
	# Copy package there
	cp $(STENCILA_R_PACKAGE) $(STENCILA_R_REPO_DIR)
	# Generate the PACKAGE file for the repo
	Rscript -e "tools::write_PACKAGES('$(STENCILA_R_REPO_DIR)',type='$(STENCILA_R_REPO_TYPE)')"
	# Make the directory for the shared library
	mkdir -p repo/$(STENCILA_R_LIB_DIR)
	# Copy the library zip file there
	cp build/$(STENCILA_R_LIB_ZIPFILE) repo/$(STENCILA_R_LIB_DIR)

# Sync the local repository with the global repository
publish:
	aws s3 sync repo s3://get.stenci.la/r

install:
	R CMD INSTALL $(STENCILA_R_PACKAGE)

clean:
	rm -rf *.o *.so *.dll stencila.Rcheck testing

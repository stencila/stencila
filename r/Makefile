include ../cpp/Makefile.common

# R does not like version numbers which are not in like X.Y.Z (or use a - instead of a .)
# So in some place use the latest tag instead of the full git generated version string
TAG := $(shell git describe --abbrev=0)

.PHONY: requirements compile rds docs desc test install

all:  test

# Requires everything that Stencila C++ library requires and the Stencila C++ library itself
requirements:
	$(MAKE) -C ../cpp/requirements all
	$(MAKE) -C ../cpp/lib all
	$(MAKE) -C requirements all

# Get list of C++ files and corresponding .o files
STENCILA_R_CPPS := $(wildcard *.cpp)
STENCILA_R_OS := $(patsubst %.cpp,%.o,$(STENCILA_R_CPPS))

# Get R and Rcpp compile flags
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

#Define the outputs
ifeq ($(STENCILA_PLATFORM),linux)
STENCILA_R_EXT := stencila_.so
STENCILA_R_PACKAGE := stencila_$(TAG).tar.gz
endif
ifeq ($(STENCILA_PLATFORM),msys)
STENCILA_R_EXT := stencila_.dll
STENCILA_R_PACKAGE := stencila_$(TAG).zip
endif


%.o : %.cpp
	$(CXX) -c -O2 -fPIC $(R_CPPFLAGS) $(RCPP_CXXFLAGS) $(STENCILA_CPP_FLAGS) $(STENCILA_CPP_INCLUDE_DIRS) -o$@ $<

$(STENCILA_R_EXT): version.hpp requirements $(STENCILA_R_OS)
	$(CXX) -shared $(STENCILA_R_OS) -o$(STENCILA_R_EXT) $(R_LDFLAGS) $(RCPP_LDFLAGS) $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS)

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
# it creates a tar.gz file but that file does not include precomiled extensions (ie .so and .dll files)
# So, the folowing line replaces R CMD build:
#   tar cfz stencila_$(TAG).tar.gz stencila
$(STENCILA_R_PACKAGE): $(STENCILA_R_EXT) rds
	rm -f stencila/src/*.o stencila/src/stencila.so stencila/src/stencila.dll #Remove any old build artifacts
	cp $(STENCILA_R_EXT) stencila/src/$(STENCILA_R_EXT) #Copy in the compiled extension
ifeq ($(STENCILA_PLATFORM),linux)
	tar cfz stencila_$(TAG).tar.gz stencila #Tarball it up
endif
ifeq ($(STENCILA_PLATFORM),msys)
	R CMD INSTALL --build stencila #Build it as per usual for windows
endif


# Deposit package into local repository
ifeq ($(STENCILA_PLATFORM),linux)
STENCILA_REPO_DIR := repo/src/contrib
STENCILA_REPO_TYPE := source
endif
ifeq ($(STENCILA_PLATFORM),msys)
STENCILA_REPO_DIR := repo/bin/windows/contrib/2.15
STENCILA_REPO_TYPE := win.binary
endif
deposit: $(STENCILA_R_PACKAGE)
	mkdir -p  $(STENCILA_REPO_DIR) #Make R package repository sub directory
	cp $(STENCILA_R_PACKAGE) $(STENCILA_REPO_DIR) #Copy package there
	Rscript -e "tools::write_PACKAGES('$(STENCILA_REPO_DIR)',type='$(STENCILA_REPO_TYPE)')" #Generate the PACKAGE file for the repo


# Generate Stencila custom HTML documentation
docs: rds
	$(MAKE) -C docs

# Check the package by running R CMD check
r_check:
	R CMD check stencila

# Test the package by running unit tests
# Install package in a testing directory and run unit tests from there
# This is better than installing package in the users R library location
unit_tests: $(STENCILA_R_PACKAGE)
	mkdir -p testing
	R CMD INSTALL -l testing $(STENCILA_R_PACKAGE)
	echo "setwd('testing'); library(stencila,lib.loc='.'); source('stencila/unitTests/do.svUnit.R')" | R --vanilla

test: r_check unit_tests

install:
	R CMD INSTALL $(STENCILA_R_PACKAGE)

clean:
	rm -rf *.o *.so *.dll stencila.Rcheck testing

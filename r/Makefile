include ../cpp/Makefile.common

# R does not like version numbers which are not in like X.Y.Z (or use a - instead of a .)
# So in some place use the latest tag instead of the full git generated version string
TAG := $(shell git describe --abbrev=0)
VERSION :=  $(shell git describe)

.PHONY: compile docs desc test install

all:  build test

# Requires everything that C++ library requires
requirements:
	make -C ../cpp/requirements all

# The compile job is for testing compilation without doing the full "R CMD check"
# The files stencila.so and stencila.o are remove to prevent R CMD check from complaining later
RCPP_CXXFLAGS := $(shell Rscript -e "Rcpp:::CxxFlags()")
RCPP_LDFLAGS :=  $(shell Rscript -e "Rcpp:::LdFlags()")
compile: requirements
	cp version.hpp.template version.hpp
	sed -i 's!version = .*;$$!version = "$(VERSION)";!' version.hpp
	g++ $(RCPP_CXXFLAGS) $(STENCILA_CPP_FLAGS) -fpic -shared \
		-I/usr/share/R/include $(STENCILA_CPP_INCLUDE_DIRS) \
		-o stencila.so stencila.cpp \
		-L/usr/lib/R/lib -lR $(RCPP_LDFLAGS) \
		$(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS)
		
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

# Build the package
# When you run 
#   R CMD build stencila
# it creates a tar.gz file but that file does not include stencila_.so
# So, the folowing line replaces it
#   tar cfz stencila_$(TAG).tar.gz stencila
build: compile docs
	mv stencila.so stencila/src/stencila_.so
	tar cfz stencila_$(TAG).tar.gz stencila
	mkdir -p builds
	mv stencila_$(TAG).tar.gz builds
	ln -sf stencila_$(TAG).tar.gz builds/stencila_latest.tar.gz 
	
test:
	R CMD check builds/stencila_$(TAG).tar.gz
	
install:
	R CMD INSTALL builds/stencila_$(TAG).tar.gz
	
clean:
	rm -rf stencila.Rcheck
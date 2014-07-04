# Include for various STENCILA_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/../variables.make

# Get the Stencila C++ package directory path.
STENCILA_CPP_HOME := $(STENCILA_HOME)/cpp

# Define compiler flags
STENCILA_CPP_FLAGS := -pthread -std=c++11 -Wall -Wno-unused-local-typedefs -DSTENCILA_HOME='"$(STENCILA_HOME)"'

# Define directories for includes
STENCILA_CPP_INCS := -I$(STENCILA_CPP_HOME) -I$(STENCILA_CPP_HOME)/requires/include

# Define directories for static libs
STENCILA_CPP_LIDS := -L$(STENCILA_CPP_HOME)/requires/lib

# Define libraries required
STENCILA_CPP_LIBS :=
STENCILA_CPP_LIBS += -lboost_filesystem -lboost_system -lboost_regex
STENCILA_CPP_LIBS += -lgit2 -lcrypto -lssl -lrt # libgit2 requires libcrypto, libssl and librt
STENCILA_CPP_LIBS += -lpugixml
STENCILA_CPP_LIBS += -ltidy-html5

# Platform specific requirements
ifeq ($(STENCILA_PLATFORM), linux)
STENCILA_CPP_LIBS +=
endif
ifeq ($(STENCILA_PLATFORM), msys)
STENCILA_CPP_LIBS +=
endif

# Compilation variables when compiling in Python
STENCILA_CPP_INCS_PY := -I/usr/include/python2.7
STENCILA_CPP_LIBS_PY := -lpython2.7 -lboost_python

# Compilation variables when compiling in R
# This assumes that Rcpp and RInside packages are installed in `/usr/lib/R/library` e.g.
# 	install.packages(c('Rcpp','RInside'),lib="/usr/lib/R/library")
# Based on running the following
#  `R CMD config --cppflags`
#  `R CMD config --ldflags`
#  `Rscript -e "Rcpp:::CxxFlags()"`
#  `Rscript -e "RInside:::CxxFlags()"`
#  `Rscript -e "RInside:::LdFlags()"`
STENCILA_CPP_FLAGS_R := -Wl,--export-dynamic -fopenmp
STENCILA_CPP_INCS_R := -I/usr/share/R/include -I/usr/lib/R/library/Rcpp/include -I/usr/lib/R/library/RInside/include
STENCILA_CPP_LIDS_R := -L/usr/lib/R/lib -L/usr/lib/R/library/RInside/lib
# Note that linking to the static verion of the RInside library by providing its absolute path
STENCILA_CPP_LIBS_R := -lR -lpcre -llzma -lbz2 -lrt -ldl -lm /usr/lib/R/library/RInside/lib/libRInside.a

# Include for various STENCILA_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/../variables.make

# Get the Stencila C++ package directory path.
STENCILA_CPP_HOME := $(STENCILA_HOME)/cpp

# Define compiler flags
STENCILA_CPP_FLAGS := -pthread -std=c++11 -Wall -Wno-unused-local-typedefs -DSTENCILA_HOME='"$(STENCILA_HOME)"'

# Define directories for includes
STENCILA_CPP_INCLUDE_DIRS := -I$(STENCILA_CPP_HOME) -I$(STENCILA_CPP_HOME)/requires/include
STENCILA_CPP_INCLUDE_DIRS += -I/usr/include/python2.7
STENCILA_CPP_INCLUDE_DIRS += -I/usr/share/R/include -I/home/nbentley/R/x86_64-pc-linux-gnu-library/3.1/Rcpp/include -I/home/nbentley/R/x86_64-pc-linux-gnu-library/3.1/RInside/include

# Define directories for static libs
STENCILA_CPP_LIB_DIRS := -L$(STENCILA_CPP_HOME)/lib -L$(STENCILA_CPP_HOME)/requires/lib

# Define libraries required
STENCILA_CPP_LIBS :=
STENCILA_CPP_LIBS += -lboost_filesystem -lboost_system -lboost_regex
STENCILA_CPP_LIBS += -lgit2 -lcrypto -lssl -lrt # libgit2 requires libcrypto, libssl and librt
STENCILA_CPP_LIBS += -lpugixml
STENCILA_CPP_LIBS += -ltidy-html5
STENCILA_CPP_LIBS += -lpython2.7 -lboost_python
# From `R CMD config --ldflags`
STENCILA_CPP_LIBS += -Wl,--export-dynamic -fopenmp  -L/usr/lib/R/lib -lR -lpcre -llzma -lbz2 -lrt -ldl -lm
# From `Rscript -e "RInside:::LdFlags()"`
STENCILA_CPP_LIBS += -L/home/nbentley/R/x86_64-pc-linux-gnu-library/3.1/RInside/lib -lRInside -Wl,-rpath,/home/nbentley/R/x86_64-pc-linux-gnu-library/3.1/RInside/lib

ifeq ($(STENCILA_PLATFORM), linux)
STENCILA_CPP_LIBS +=
endif
ifeq ($(STENCILA_PLATFORM), msys)
STENCILA_CPP_LIBS +=
endif

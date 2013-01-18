include $(dir $(lastword $(MAKEFILE_LIST)))/../stencila.mk

#Get the Stencila C++ library directory path.
STENCILA_CPP_HOME := $(STENCILA_HOME)/cpp

#Define compiler flags required
STENCILA_CPP_FLAGS := -std=c++0x -Wall

#Define directories for includes and libs
STENCILA_CPP_INCLUDE_DIRS := -I$(STENCILA_CPP_HOME) -I$(STENCILA_CPP_HOME)/requirements/include
STENCILA_CPP_LIB_DIRS := -L$(STENCILA_CPP_HOME)/lib -L$(STENCILA_CPP_HOME)/requirements/lib

#Define libraries required
# Note that -ldl -pthread should come after -lsqlite
STENCILA_CPP_LIBS := -lstencila -lboost_system -lboost_filesystem -lboost_date_time -lsqlite3 -lpugixml
ifeq ($(STENCILA_PLATFORM), linux)
STENCILA_CPP_LIBS := $(STENCILA_CPP_LIBS) -ldl -lpthread
endif

#Task for updating version.hpp
version.hpp:
	cd $(STENCILA_CPP_HOME)/stencila/; \
	cp version.hpp.template version.hpp; \
	sed -i 's!version = .*;$$!version = "$(STENCILA_VERSION)";!' version.hpp

.PHONY: version.hpp

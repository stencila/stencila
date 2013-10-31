include $(dir $(lastword $(MAKEFILE_LIST)))/../stencila.mk

#Get the Stencila C++ library directory path.
STENCILA_CPP_HOME := $(STENCILA_HOME)/cpp

#Define compiler flags required
STENCILA_CPP_FLAGS := -std=c++0x -Wall -DSTENCILA_HOME='"$(STENCILA_HOME)"'
ifeq ($(STENCILA_PLATFORM), msys)
#On MSYS define _WIN32_WINNT to prevent boost::asio from complaining that it is not defined
#0x0501 is the default value that boost::asio uses if it is missing so it assumed to be OK to use in most(?) situations
STENCILA_CPP_FLAGS += -D _WIN32_WINNT=0x0501
endif

#Define directories for includes and libs
STENCILA_CPP_INCLUDE_DIRS := -I$(STENCILA_CPP_HOME) -I$(STENCILA_CPP_HOME)/requirements/include

STENCILA_CPP_LIB_DIRS := -L$(STENCILA_CPP_HOME)/lib -L$(STENCILA_CPP_HOME)/requirements/lib
ifeq ($(STENCILA_PLATFORM), msys)
STENCILA_CPP_LIB_DIRS += -L/usr/lib
endif

#Define libraries required
# Note that -ldl -pthread should come after -lsqlite
STENCILA_CPP_LIBS := -lstencila -lboost_system -lboost_filesystem -lboost_date_time -lboost_thread -lcppnetlib-client-connections -lcppnetlib-server-parsers -lcppnetlib-uri -lsqlite3 -lpugixml -ltidy-html5 -lssl -lcrypto -larchive -lz
ifeq ($(STENCILA_PLATFORM), linux)
STENCILA_CPP_LIBS += -ldl -lpthread
endif
ifeq ($(STENCILA_PLATFORM), msys)
STENCILA_CPP_LIBS += -lws2_32 -lkernel32
endif

#Task for updating version.hpp
version.hpp:
	cd $(STENCILA_CPP_HOME)/stencila/; \
	cp version.hpp.template version.hpp; \
	sed -i 's!version = .*;$$!version = "$(STENCILA_VERSION)";!' version.hpp

.PHONY: version.hpp

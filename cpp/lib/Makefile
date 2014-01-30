all: libstencila.a

.PHONY: all clean

# Include for various STENCILA_XXX and STENCILA_CPP_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/../variables.make

# Define a list of CPP files by recursively searching source directory
STENCILA_CPPS := $(shell find ../stencila -name "*.cpp")
# Strip off directory names and replace .cpp with .o
STENCILA_OS := $(patsubst %.cpp,%.o,$(notdir $(STENCILA_CPPS)))

# Set VPATH for make to search for prerequisites
# http://www.gnu.org/software/make/manual/make.html#General-Search
VPATH = ../stencila ../stencila/utilities

%.o : %.cpp
	$(CXX) $(STENCILA_CPP_FLAGS) $(STENCILA_CPP_INCLUDE_DIRS) -fPIC -c -O3 -o$@ $<

libstencila.a: $(STENCILA_OS)
	$(AR) rcs $@ $(STENCILA_OS)

clean:
	rm -f *.o *.a

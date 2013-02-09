include $(dir $(lastword $(MAKEFILE_LIST)))/../cpp.mk

STENCILA_CPPS := $(shell find ../stencila -name *.cpp)
STENCILA_OS := $(patsubst ../stencila/%.cpp,%.o,$(STENCILA_CPPS))

all: libstencila.a

%.o : ../stencila/%.cpp
	$(CXX) $(STENCILA_CPP_FLAGS) $(STENCILA_CPP_INCLUDE_DIRS) -c -O2 -o $@ $<

libstencila.a: $(STENCILA_OS)
	$(AR) rc $@ *.o

clean:
	rm -f *.o *.a

all: libstencila.a

include $(dir $(lastword $(MAKEFILE_LIST)))/../cpp.mk

STENCILA_CPPS := $(shell find ../stencila -name "*.cpp")
STENCILA_OS := $(patsubst %.cpp,%.o,$(STENCILA_CPPS))

%.o : %.cpp
	$(CXX) $(STENCILA_CPP_FLAGS) $(STENCILA_CPP_INCLUDE_DIRS) -fPIC -c -O2 -o $@ $<

libstencila.a: $(STENCILA_OS)
	$(AR) rc $@ $(STENCILA_OS)

clean:
	rm -f *.a

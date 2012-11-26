include ../cpp/Makefile.common

.PHONY: build test clean

all: build test
	
build: version.hpp
	g++ $(STENCILA_CPP_FLAGS) -shared -fPIC -O3 $(STENCILA_CPP_INCLUDE_DIRS) -ostencila.so stencila.cpp

test:
	make -C tests all
	
clean:
	rm -f stencila.so

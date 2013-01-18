include ../cpp/cpp.mk

.PHONY: build test clean

all: build test
	
build: version.hpp
	$(CXX) $(STENCILA_CPP_FLAGS) -shared -fPIC -O2 $(STENCILA_CPP_INCLUDE_DIRS) -ostencila.so stencila.cpp \
		$(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS)

test:
	make -C tests all
	
clean:
	rm -f stencila.so

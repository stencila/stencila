.PHONY: build test clean

all: build test
	
build:
	gcc -shared -fPIC -lm -I../../../.. -ostencila.so stencila.c
	
test:
	make -C tests all
	
clean:
	rm -f stencila.so

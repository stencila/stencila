all: stencila.so test
	
stencila.so: stencila.c
	gcc -shared -fPIC -lm -I../../../.. -ostencila.so stencila.c
	
test:
	make -C tests all

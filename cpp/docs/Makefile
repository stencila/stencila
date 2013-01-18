include $(dir $(lastword $(MAKEFILE_LIST)))/../../stencila.mk

all:
	#Using sed update PROJECT_NUMBER in Doxyfile
	#	.* = anything, any number of times
	#	$ = end of line
	#The $ needs to be doubled for escaping make
	cp Doxyfile.template Doxyfile
	sed -i 's!PROJECT_NUMBER = .*$$!PROJECT_NUMBER = $(STENCILA_VERSION)!' Doxyfile
	doxygen Doxyfile
	
clean:
	rm -rf html

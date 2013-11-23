include $(dir $(lastword $(MAKEFILE_LIST)))/../../stencila.mk

.PHONY: html

all: html

html:
	#Using sed update PROJECT_NUMBER in Doxyfile
	#	.* = anything, any number of times
	#	$ = end of line
	#The $ needs to be doubled for escaping make
	cp Doxyfile.template Doxyfile
	sed -i 's!PROJECT_NUMBER = .*$$!PROJECT_NUMBER = $(STENCILA_VERSION)!' Doxyfile
	doxygen Doxyfile

publish:
	aws s3 sync html s3://docs.stenci.la/cpp
	
clean:
	rm -rf html

#Get version number from git
VERSION = $(shell git describe)

all:
	#Using sed update PROJECT_NUMBER in Doxyfile
	#	.* = anything, any number of times
	#	$ = end of line
	#The $ needs to be doubled for escaping make
	cp Doxyfile.template Doxyfile
	sed -i 's!PROJECT_NUMBER = .*$$!PROJECT_NUMBER = $(VERSION)!' Doxyfile
	doxygen Doxyfile
	
clean:
	rm -rf html
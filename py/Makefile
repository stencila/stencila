.PHONY: build dist test clean

all: build dist test

#Separate build step
build:
	python2.7 setup.py build
	
#Create binary distribtion for the platform
dist: build
	python2.7 setup.py bdist_egg
	
#Install in tests virtual environment.
#Create a virtual environment first, activate it and then install, then run tests
test: dist
	virtualenv tests/env; \
	. tests/env/bin/activate ;\
	python setup.py install ;\
	make -C tests all ;
	
install:
	sudo python2.7 setup.py install
	
clean:
	rm -rf build dist tests/env
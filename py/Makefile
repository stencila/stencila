.PHONY: all dist build install tests

all: dist build install tests

#Create binary distribtion for the platform
dist:
	python2.7 setup.py bdist

#Separate build step
build:
	python2.7 setup.py build
	
#Install in tests virtual environment.
#Create a virtual environment first and then specify it as install prefix
install:
	virtualenv tests/env; \
	python2.7 setup.py install --prefix=tests/env; \
	
tests:
	make -C tests all
	
clean:
	rm -rf build dist tests/env
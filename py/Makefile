include ../cpp/Makefile.common

.PHONY: build dist test clean

all: clean test

# Requires everything that C++ library requires
requirements:
	make -C ../cpp/requirements all

# Make taks then get called by setup.py
cpp_flags:
	@echo $(STENCILA_CPP_FLAGS)
cpp_include_dirs:
	@echo $(STENCILA_CPP_INCLUDE_DIRS)
cpp_library_dirs:
	@echo $(STENCILA_CPP_LIB_DIRS)
cpp_libs:
	@echo $(STENCILA_CPP_LIBS)

#Separate build step
build: requirements
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
	python2.7 setup.py install
	
clean:
	rm -rf build dist tests/env
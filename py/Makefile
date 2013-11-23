
all: test

.PHONY: build dist test clean

include ../cpp/cpp.mk

# Requires everything that Stencila C++ library requires and the Stencila C++ library itself
requirements:
	make -C ../cpp/requirements all
	make -C ../cpp/lib all

# Tasks that get called by setup.py to return variables
version:
	@echo $(STENCILA_VERSION)
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
	python setup.py build
	
#Create binary distribution for the platform
dist: build
	python setup.py bdist_egg

#Publish distribution
publish:
	aws s3 sync dist s3://get.stenci.la/py
	
#Install in tests virtual environment.
#Create a virtual environment first, activate it and then install, then run tests
test: dist
	virtualenv tests/env; \
	. tests/env/bin/activate ;\
	python setup.py install ;\
	make -C tests all ;
	
install:
	python setup.py install
	
clean:
	rm -rf build dist tests/env

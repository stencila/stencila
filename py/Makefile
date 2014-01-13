all: test

.PHONY: build dist test

include $(dir $(lastword $(MAKEFILE_LIST)))/../cpp/variables.make

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

# Requires everything that Stencila C++ library requires and the Stencila C++ library itself
requires:
	$(MAKE) -C ../cpp/lib all

# Create binary distribution for the platform
dist:
	python setup.py bdist_egg

# Publish distribution
publish:
	aws s3 sync dist s3://get.stenci.la/py
	
# Create a virtual environment, activate it, install package into it and then run tests
test: dist testing
	. testing/bin/activate ;\
	python setup.py install ;\
	python tests/tests.py;

testing:
	virtualenv testing

# Clean up
clean:
	rm -rf stencila.egg-info build dist testing

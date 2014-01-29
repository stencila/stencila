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
	$(MAKE) -C ../cpp/requires all
	$(MAKE) -C ../cpp version.hpp

# Create binary distribution for the platform
# Create a text file with name of latest wheel for testing (using a symlink to latest wheel instead
# failed when installing using pip install)
dist:
	python setup.py bdist_wheel
	cd dist; echo `ls -rt *.whl | tail -n1` > latest_wheel.txt

# Publish distribution
publish:
	aws s3 sync dist s3://get.stenci.la/py
	
# Create a virtual environment, activate it, install package into it and then run tests
test: dist testing
	. testing/bin/activate ;\
	pip install --upgrade --force-reinstall dist/`cat dist/latest_wheel.txt`;\
	python tests/tests.py;

testing:
	virtualenv --no-site-packages testing

# Clean up
clean:
	rm -rf stencila.egg-info build dist testing

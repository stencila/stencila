all: test

.PHONY: build dist test

include $(dir $(lastword $(MAKEFILE_LIST)))/../cpp/variables.make

# Tasks that get called by setup.py to return variables
version:
	@echo $(STENCILA_VERSION)
cpp_files:
	@echo $(shell find stencila -name "*.cpp" -follow) # Follow symlinks
cpp_flags:
	@echo $(STENCILA_CPP_FLAGS)
cpp_include_dirs:
	@echo $(STENCILA_CPP_INCS)
cpp_library_dirs:
	@echo $(STENCILA_CPP_LIDS)
cpp_libs:
	@echo $(STENCILA_CPP_LIBS) -lboost_python

# Requires everything that Stencila C++ library requires
requires:
	$(MAKE) -C ../cpp/requires

# Copy PythonContext code from the `cpp` module. Append
# a hash to it to escapr the raw string.
pycode:
	echo -n "#" | cat - ../cpp/stencila/python-context.py >> stencila/context.py

# Create binary distribution for the platform
# Create a text file with name of latest wheel for testing (using a symlink to latest wheel instead
# failed when installing using pip install)
dist: requires pycode
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

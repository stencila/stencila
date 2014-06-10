all: version.hpp requires tests

.PHONY: version.hpp requires tests

# Include for various STENCILA_XXX and STENCILA_CPP_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/variables.make

# Make the version file
version.hpp:
	cd $(STENCILA_CPP_HOME)/stencila/; \
	cp version.hpp.template version.hpp; \
	sed -i 's!version = .*;$$!version = "$(STENCILA_VERSION)";!' version.hpp

# Make all the requirements
requires:
	$(MAKE) -C requires all

# Run all tests
tests:
	$(MAKE) -C tests all
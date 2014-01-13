all: version.hpp

.PHONY: version.hpp

# Include for various STENCILA_XXX and STENCILA_CPP_XXX variables
include $(dir $(lastword $(MAKEFILE_LIST)))/variables.make

version.hpp:
	cd $(STENCILA_CPP_HOME)/stencila/; \
	cp version.hpp.template version.hpp; \
	sed -i 's!version = .*;$$!version = "$(STENCILA_VERSION)";!' version.hpp

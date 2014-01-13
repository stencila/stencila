all: cpp py r

.PHONY: cpp py r

include $(dir $(lastword $(MAKEFILE_LIST)))/variables.make

version:
	@echo $(STENCILA_VERSION)

cpp:
	$(MAKE) -C cpp all
	
py:
	$(MAKE) -C py all
	
r:
	$(MAKE) -C r all

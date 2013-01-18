include ../../stencila.mk

.PHONY: all

all:
	mkdir -p html
	python r.py

# Required by r.py for getting Stencila version
version:
	@echo $(STENCILA_VERSION)
	
clean:
	rm -rf html

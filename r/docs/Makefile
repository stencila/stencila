include ../../stencila.mk

.PHONY: all

# Required by r.py for getting Stencila version
version:
	@echo $(STENCILA_VERSION)

all:
	mkdir -p html
	python r.py
	
clean:
	rm -rf html

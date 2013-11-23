include $(dir $(lastword $(MAKEFILE_LIST)))/../../stencila.mk

.PHONY: html

all: html

html:
	mkdir -p html
	python r.py

publish:
	aws s3 sync html s3://docs.stenci.la/r

# Required by r.py for getting Stencila version
version:
	@echo $(STENCILA_VERSION)
	
clean:
	rm -rf html

.PHONY : requirements lib docs tests

all: requirements lib docs tests

requirements:
	$(MAKE) -C requirements all
	
lib:
	$(MAKE) -C lib all
	
docs:
	$(MAKE) -C docs all

tests:
	$(MAKE) -C tests all

include $(dir $(lastword $(MAKEFILE_LIST)))/../stencila.mk

STENCILA_CPP_TARGZ := stencila-cpp-$(STENCILA_VERSION).tar.gz
STENCILA_CPP_ZIP := stencila-cpp-$(STENCILA_VERSION).zip

dist: dist/STENCILA_CPP_TARGZ dist/STENCILA_CPP_ZIP

dist/STENCILA_CPP_TARGZ:
	mkdir -p dist
	cd ..; git archive master:cpp | gzip > cpp/dist/$(STENCILA_CPP_TARGZ)

dist/STENCILA_CPP_ZIP:
	mkdir -p dist
	cd ..; git archive --format zip --output cpp/dist/$(STENCILA_CPP_ZIP) master:cpp

publish:
	aws s3 sync dist s3://get.stenci.la/cpp

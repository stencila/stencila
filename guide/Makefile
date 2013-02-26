include ../stencila.mk

all: output

# Required by r.py for getting Stencila version
version:
	@echo $(STENCILA_VERSION)

prepare_cpp:
	#Create working directory
	mkdir -p working/cpp
	#Make C++ requirements
	make -C ../cpp/requirements all
	#Make C++ library
	make -C ../cpp/lib all

prepare_py:
	#Create working directory
	mkdir -p working/py
	#Create a Python virtualenv and install stencila into it
	virtualenv working/py/env;
	. working/py/env/bin/activate ;	cd ../py ; python setup.py install

prepare_r:
	#Create working directory
	mkdir -p working/r
	#Install R package in local directory
	#library(stencila,lib.loc='.') is then used later used inside R scripts
	cd ../r ; make package deposit
	cd working/r ; R CMD INSTALL -l . ../../../r/stencila_$(STENCILA_VERSION).tar.gz

output: prepare_cpp prepare_py prepare_r
	python guide.py

clean:
	rm -rf working html

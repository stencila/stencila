all: output

prepare_cpp:
	#Create working directory
	mkdir -p working/cpp
	#Make C++ requirements
	make -C ../../cpp/requirements all

prepare_py:
	#Create working directory
	mkdir -p working/py
	#Create a Python virtualenv and install stencila into it
	virtualenv working/py/env;
	. working/py/env/bin/activate ;	cd ../../py ; python setup.py install

prepare_r:
	#Create working directory
	mkdir -p working/r
	#Install R package in local directory
	#library(stencila,lib.loc='.') is then used later used inside R scripts
	cd ../../r ; make build
	cd working/r ; R CMD INSTALL -l . ../../../../r/builds/stencila_latest.tar.gz

output: prepare_cpp prepare_py prepare_r
	python guide.py

clean:
	rm -rf working html
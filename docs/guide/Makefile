all: output
	
prepare_py:
	#Create a Python virtualenv and install stencila into it
	virtualenv working/py/env; \
	. working/py/env/bin/activate ;\
	cd ../../py ; python setup.py install
	
prepare_r:
	#Install R package in local directory
	#library(stencila,lib.loc='.') is then used later used inside R scripts
	cd ../../r ; make build
	cd working/r ; R CMD INSTALL -l . ../../../../r/builds/stencila_latest.tar.gz
	
output: prepare_py prepare_r
	python guide.py
	
clean:
	rm -rf html
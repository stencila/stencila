all: output
	
prepare_py:
	#Create a Python virtualenv and install stencila into it
	virtualenv working/py/env; \
	. working/py/env/bin/activate ;\
	cd ../../py ; python setup.py install
	
prepare_r:
	#Install R package globally. Not ideal but does the job for now
	cd ../../r ; make install
	
output: prepare_py prepare_r
	python guide.py
	
clean:
	rm -rf html
.PHONY: cpp py r sqlite guide

all: cpp py r sqlite guide

cpp:
	make -C cpp all
	
py:
	make -C py all
	
r:
	make -C r all
	
sqlite:
	make -C sqlite all
	
guide:
	make -C guide all


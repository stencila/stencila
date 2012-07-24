.PHONY: all cpp py r sqlite docs

all: cpp py r sqlite docs

cpp:
	make -C cpp all
	
py:
	make -C py all
	
r:
	make -C r all
	
sqlite:
	make -C sqlite all
	
docs:
	make -C docs all


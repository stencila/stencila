.PHONY: all cpp docs

all: cpp docs

cpp:
	make -C cpp all
	
docs:
	make -C docs all


all: cpp docs

cpp:
	make -C cpp all
	
docs:
	make -C docs all
	
.PHONY: all cpp docs

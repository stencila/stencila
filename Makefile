all: cxx docs

cxx:
	make -C cxx all
	
docs:
	make -C docs all
	
.PHONY: all cxx docs

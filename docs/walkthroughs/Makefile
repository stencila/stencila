# Makefile for generating walkthroughs from source walkthroughs in SMD. 
# Not all walkthroughs can be generated like this because they may have
# syntax specific prose.

# Track just smd's we can walk over to myst (some have to be hand converted)
SOURCES = math.smd mermaid.smd aicommands.smd

OUTMYST = $(SOURCES:.smd=.myst)

all: $(OUTMYST)

clean: 
	rm $(OUTMYST)

%.myst: %.smd DUMMY
	cargo run -p cli convert $< $@ 

# A dummy target to force rebuilds
DUMMY:

# Makefile for generating walkthoughs from source walkthroughs in SMD. 
# Not all walkthroughs can be generated like this because they may have
# syntax specific prose.

all: math.myst mermaid.myst aicommands.myst

%.myst: %.smd DUMMY
	cargo run -p cli convert $< $@ 

# A dummy target to force rebuilds
DUMMY:

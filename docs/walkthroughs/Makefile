# Makefile for generating walkthroughs from source walkthroughs in SMD. 
# Not all walkthroughs can be generated like this because they may have
# syntax specific prose.

SOURCES = math mermaid
TARGETS = $(foreach src,$(SOURCES),$(src).myst $(src).qmd)

all: $(TARGETS)

%.myst %.qmd: %.smd
	cargo run -p cli convert $< $@

# Makefile for downloading OpenAlex API examples
# Downloads random examples of works, authors, and institutions from OpenAlex API

# Define entity types and their counts
ENTITIES := work author institution
NUMBERS := 1 2 3

# Generate all target files
WORK_FILES := $(foreach n,$(NUMBERS),work-$(n).json)
AUTHOR_FILES := $(foreach n,$(NUMBERS),author-$(n).json)
INSTITUTION_FILES := $(foreach n,$(NUMBERS),institution-$(n).json)
ALL_FILES := $(WORK_FILES) $(AUTHOR_FILES) $(INSTITUTION_FILES)

.PHONY: all works authors institutions clean help

# Download all example types
all: $(ALL_FILES)

# Download specific entity types
works: $(WORK_FILES)
authors: $(AUTHOR_FILES)
institutions: $(INSTITUTION_FILES)

# Pattern rules for downloading entities
work-%.json:
	curl -s "https://api.openalex.org/works/random" | jq '.' > $@

author-%.json:
	curl -s "https://api.openalex.org/authors/random" | jq '.' > $@

institution-%.json:
	curl -s "https://api.openalex.org/institutions/random" | jq '.' > $@

# Additional examples

# This article has the highest citations (at the time of writing) and so often
# appears during manual testing. However, it has some quirks such as `display_name`
# having initials concatenated with given name e.g. "OliverH. Lowry". So this,
# example is added just to identify the fact that such issues are not related to
# errors in our decoding in this crate
work-4.json:
	curl -s "https://api.openalex.org/works/W1775749144" | jq '.' > $@

# Clean up downloaded files
clean:
	rm -f *.json

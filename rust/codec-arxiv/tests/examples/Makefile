# Downloads examples of preprint HTML from arXiv for testing
#
# The following are recent examples (at time of writing, July 2025) chosen
# haphazardly from arXiv for a variety of content types (e.g. figures, tables,
# maths, appendices, conversion errors) and subject areas.

EXAMPLES := 2507.09057v1 2507.11127v1 2507.11353v1 2507.11538v1
HTML_FILES := $(addsuffix .html, $(EXAMPLES))

all: $(HTML_FILES)

%.html:
	curl https://arxiv.org/html/$* -o $@

clean:
	rm -f $(HTML_FILES)

.PHONY: all clean

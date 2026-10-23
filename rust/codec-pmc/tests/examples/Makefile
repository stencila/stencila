# Downloads examples of PMC Open Access articles for testing
#
# The following are examples chosen for a variety of content types (e.g. figures, tables,
# maths, appendices, conversion errors) and subject areas. The same PMC IDs are used for
# both HTML and tar.gz formats to allow comparison of parsing results.
# The committed tar.gz files are legacy fixtures: PMC retired their download
# service in August 2026. Preserve them for backwards compatibility tests.
# AWS Open Data fixtures are in ../bucket/.

# PMC IDs for examples (used for both HTML and tar.gz)
EXAMPLES := PMC11518443 PMC12354728 PMC12419688 PMC12431436
HTML_FILES := $(addsuffix .html, $(EXAMPLES))

all: $(HTML_FILES)

# Download HTML versions from PMC
%.html:
	curl -A "Stencila-Tests"  https://pmc.ncbi.nlm.nih.gov/articles/$*/ -o $@

clean:
	rm -f $(HTML_FILES)

.PHONY: all clean

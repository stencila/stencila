# Downloads examples of PMC Open Access articles for testing
#
# The following are examples chosen for a variety of content types (e.g. figures, tables,
# maths, appendices, conversion errors) and subject areas. The same PMC IDs are used for
# both HTML and tar.gz formats to allow comparison of parsing results.

# PMC IDs for examples (used for both HTML and tar.gz)
EXAMPLES := PMC11518443 PMC12354728 PMC12419688 PMC12431436
HTML_FILES := $(addsuffix .html, $(EXAMPLES))
TAR_FILES := $(addsuffix .tar.gz, $(EXAMPLES))

all: $(HTML_FILES) $(TAR_FILES)

# Download HTML versions from PMC
%.html:
	curl -A "Stencila-Tests"  https://pmc.ncbi.nlm.nih.gov/articles/$*/ -o $@

# Download tar.gz packages from PMC FTP
# Get the URL from https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi?id=PMC12419688

PMC11518443.tar.gz:
	curl -O https://ftp.ncbi.nlm.nih.gov/pub/pmc/oa_package/08/01/PMC11518443.tar.gz

PMC12354728.tar.gz:
	curl -O https://ftp.ncbi.nlm.nih.gov/pub/pmc/oa_package/67/5d/PMC12354728.tar.gz

PMC12419688.tar.gz:
	curl -O https://ftp.ncbi.nlm.nih.gov/pub/pmc/oa_package/e4/9c/PMC12419688.tar.gz

PMC12431436.tar.gz:
	curl -O https://ftp.ncbi.nlm.nih.gov/pub/pmc/oa_package/89/87/PMC12431436.tar.gz

clean:
	rm -f $(HTML_FILES) $(TAR_FILES)

.PHONY: all clean

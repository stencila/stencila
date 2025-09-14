# Downloads examples of PMC Open Access articles for testing
#
# The following are examples chosen for a variety of content types (e.g. figures, tables,
# maths, appendices, conversion errors) and subject areas. The same PMC IDs are used for
# both HTML and tar.gz formats to allow comparison of parsing results.

# PMC IDs for examples (used for both HTML and tar.gz)
EXAMPLES := PMC11518443 PMC12431436
HTML_FILES := $(addsuffix .html, $(EXAMPLES))
TAR_FILES := $(addsuffix .tar.gz, $(EXAMPLES))

all: $(HTML_FILES) $(TAR_FILES)

# Download HTML versions from PMC
%.html:
	curl -A "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36"  https://pmc.ncbi.nlm.nih.gov/articles/$*/ -o $@

# Download tar.gz packages from PMC FTP
# Get the URL from https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi?id=PMC12431436

PMC11518443.tar.gz:
	curl -O https://ftp.ncbi.nlm.nih.gov/pub/pmc/oa_package/08/01/PMC11518443.tar.gz

PMC12431436.tar.gz:
	curl -O https://ftp.ncbi.nlm.nih.gov/pub/pmc/oa_package/89/87/PMC12431436.tar.gz

clean:
	rm -f $(HTML_FILES) $(TAR_FILES)

.PHONY: all clean

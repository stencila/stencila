# Makefile for downloading Zenodo API examples

NUMBERS := 1 2 3

SEARCH_RECORDS_FILES := $(foreach n,$(NUMBERS),search-records-$(n).json)
RECORD_FILES := $(foreach n,$(NUMBERS),record-$(n).json)

ALL_FILES := $(SEARCH_RECORDS_FILES) $(RECORD_FILES)

ZENODO_API := https://zenodo.org/api

define CURL_CMD
curl -s \
  -H "Accept: application/json" \
  $(if $(ZENODO_TOKEN),-H "Authorization: Bearer $(ZENODO_TOKEN)") \
  $(1) | jq '.'
endef

.PHONY: all search-records records

# Download all example types
all: $(ALL_FILES)

# Download specific entity types
search-records: $(SEARCH_RECORDS_FILES)
records: $(RECORD_FILES)

# Pattern rules for downloading search records responses
# These search for different types of content on Zenodo
search-records-1.json:
	$(call CURL_CMD,"$(ZENODO_API)/records?q=python+dataset&size=1") > $@

search-records-2.json:
	$(call CURL_CMD,"$(ZENODO_API)/records?q=type:software&size=1") > $@

search-records-3.json:
	$(call CURL_CMD,"$(ZENODO_API)/records?q=type:publication&size=1") > $@

# Pattern rules for downloading individual record responses
# These are specific interesting records from Zenodo
record-1.json:
	# A popular dataset: "Iris Dataset" (record ID: 5574486)
	$(call CURL_CMD,"$(ZENODO_API)/records/5574486") > $@

record-2.json:
	# A software record: scikit-learn (record ID: 11189141)
	$(call CURL_CMD,"$(ZENODO_API)/records/11189141") > $@

record-3.json:
	# A publication record (record ID: 8154610)
	$(call CURL_CMD,"$(ZENODO_API)/records/8154610") > $@

# Clean up downloaded files
clean:
	rm -f *.json

# Download specific records by ID (for testing)
# Usage: make record ID=12345678
record:
	@if [ -z "$(ID)" ]; then \
		echo "Usage: make record ID=<record_id>"; \
		exit 1; \
	fi
	$(call CURL_CMD,"$(ZENODO_API)/records/$(ID)") > record-$(ID).json
	@echo "Downloaded record $(ID) to record-$(ID).json"

# Search Zenodo with custom query
# Usage: make search Q="python machine learning"
search:
	@if [ -z "$(Q)" ]; then \
		echo "Usage: make search Q=\"<search query>\""; \
		exit 1; \
	fi
	$(call CURL_CMD,"$(ZENODO_API)/records?q=$(Q)&size=1") > search-custom.json
	@echo "Search results saved to search-custom.json"
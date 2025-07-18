# Downloads examples of CSL-JSON from https://doi.org for testing

EXAMPLES := 10.1093/ajae/aaq063 \
            10.48550/arxiv.2507.11353 \
			10.1101/2025.06.27.662068

JSON_FILES := $(addsuffix .json, $(subst /,-,$(EXAMPLES)))

all: $(JSON_FILES)

%.json:
	$(eval DOI := $(subst -,/,$*))
	curl -L https://doi.org/$(DOI) -o $@ -H Accept:application/vnd.citationstyles.csl+json

clean:
	rm -f $(JSON_FILES)

.PHONY: all clean

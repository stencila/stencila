# Downloads examples of CSL-JSON from https://doi.org for testing

EXAMPLES := 10.1093/ajae/aaq063 \
            10.48550/arxiv.2507.11353 \
			10.1101/2025.06.27.662068 \
			10.1126/science.abn7950 \
			10.1371/journal.pone.0295623 \
			10.1016/j.cell.2024.01.009 \
			10.4060/cb0365en

JSON_FILES := $(addsuffix .json, $(subst /,-,$(EXAMPLES)))

all: $(JSON_FILES)

%.json:
	$(eval DOI := $(subst -,/,$*))
	curl -sL https://doi.org/$(DOI) -H Accept:application/vnd.citationstyles.csl+json | jq '.' > $@

clean:
	rm -f $(JSON_FILES)

.PHONY: all clean

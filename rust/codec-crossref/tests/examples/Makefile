# Downloads examples of responses from https://api.crossref.org/works for testing deserialization

BASE_URL:="https://api.crossref.org/works"

all: work-1.json \
	 work-list-1.json \
	 work-list-2.json

work-1.json:
	curl -sL "$(BASE_URL)/10.1139/f2011-143" | jq '.' > $@

work-list-1.json:
	curl -sL "$(BASE_URL)?query=Feasible+stock+trajectories&rows=10" | jq '.' > $@

clean:
	rm -f *.json

.PHONY: all clean

all: elife-small.json elife-mid.json

# A smallish eLife article useful for things like visual
# regression tests
elife-small.json:
	encoda convert https://elifesciences.org/articles/60912 $@

# A mid-sized eLife article with several figures and tables
elife-mid.json:
	encoda convert https://elifesciences.org/articles/67409 $@

clean:
	rm -rf *.json*

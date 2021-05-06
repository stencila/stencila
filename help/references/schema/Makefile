all:
	make fetch
	make slugify

fetch:
	rm -rf docs
	curl -sL https://github.com/stencila/schema/releases/latest/download/docs.tar.gz | tar -xz

docs/%.slugify: 
	sed -i '/^category: .*/a slug: /schema/$*' docs/$*.md
slugify: $(patsubst docs/%.md,docs/%.slugify,$(wildcard docs/*.md))

clean:
	rm -rf docs

all:
	curl -sL https://github.com/stencila/schema/releases/latest/download/docs.tar.gz | tar --strip-components=1 -xvz
	cat Introduction.md.tpl index.md > Introduction.md; rm index.md
	mv code Code
	mv data Data
	mv media Media
	mv other Other
	mv prose Prose

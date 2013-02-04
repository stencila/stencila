all: stencila.min.comb.js stencila.min.css

.PHONY: requirements

requirements:
	bower install \
		git://github.com/KenPowers/Base.js-Module.git\
		font-awesome#3.0.2\
		"http://ie7-js.googlecode.com/svn/version/2.1(beta4)/IE9.js"\
		jquery#1.9.0\
		less.js#1.3.3\
		modernizr#2.6.2\
		normalize-css#2.1.0\
		transparency#0.7.11\
		uglify-js#1.3.4 \

%.min.css: %.less
	./components/less.js/bin/lessc -x $< > $@
	
%.min.js: %.js
	./components/uglify-js/bin/uglifyjs stencila.js > stencila.min.js
	
stencila.min.comb.js: stencila.min.js
	rm -rf stencila.min.comb.js
	for filename in \
		components/jquery/jquery.min.js  \
		stencila.min.js \
	; do (cat "$${filename}"; echo "\n") >> stencila.min.comb.js ; done

all: defaults

requirements:
	bower update

# Build the default, minified JS and CSS for integration into Stencila components
defaults: requirements stencil.min.css stencil.min.js

clean:
	rm -rf *.min.js *.min.css

# Compile stencil.js
stencil.cat.js:
	cat boot.js stencil.js > stencil.cat.js

# Minify JS
%.min.js: %.cat.js
	./bower_components/uglify-js/bin/uglifyjs $< > $@

# Minify CSS
%.min.css: %.css
	./bower_components/UglifyCSS/uglifycss $< > $@
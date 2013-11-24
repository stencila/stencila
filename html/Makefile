all: defaults

# Build the default, minified JS and CSS for integration into Stencila components
defaults: stencil-default.min.css stencil-default.min.js

clean:
	rm -rf *.min.js *.min.css

# Minify JS
%.min.js: %.js
	./bower_components/uglify-js/bin/uglifyjs $< > $@

# Minify CSS
%.min.css: %.css
	./bower_components/UglifyCSS/uglifycss $< > $@
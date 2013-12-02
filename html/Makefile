all: stencil.min.css stencil.min.js

clean:
	rm -rf build *.cat.js *.min.js *.min.css

requirements:
	bower update

# Create a build directory
build:
	mkdir -p build

# Compile stencil.js
stencil.cat.js: boot.js stencil.js
	cat boot.js stencil.js > $@

# Minify JS
%.min.js: %.cat.js build
	./bower_components/uglify-js/bin/uglifyjs $< > $@
	cp $@ build

# Minify CSS
%.min.css: %.css build
	./bower_components/UglifyCSS/uglifycss $< > $@
	cp $@ build

# Publish to S3
publish: build
	aws s3 sync build s3://get.stenci.la/html
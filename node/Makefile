all: build pack test cover

.PHONY: build pack test cover

# Install dependencies
# The packages `yamljs` and `nunjucks` are in `package.json`
# but need to be installed before `wrap.js` is run
setup:
	npm install yamljs nunjucks

# Build extension
build:
	mkdir -p .wrap & node wrap.js
	npm install --build-from-source

# Create package
package:
	./node_modules/.bin/node-pre-gyp package

# Publish package
publish:
	export node_pre_gyp_accessKeyId="$$AWS_ACCESS_KEY" ;\
	export node_pre_gyp_secretAccessKey="$$AWS_SECRET_KEY" ;\
	./node_modules/.bin/node-pre-gyp publish

# Run tests
test:
	npm test

# Run tests with coverage
cover:
	npm run cover

# Install package globally
install:
	npm install -g

# Clean up
clean:
	rm -rf build coverage .wrap

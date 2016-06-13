include ../shared.make

all: build

.PHONY: build

VERSION = $(shell ../version.sh node)
NODE_VERSION = $(shell node --version)

# Install dependencies
# The packages `yamljs` and `nunjucks` are in `package.json`
# but need to be installed before `wrap.js` is run
setup:
	npm install yamljs nunjucks node-pre-gyp

# Build extension
build:
	sed -i 's!"version": .*$$!"version": "$(VERSION)",!' package.json
	mkdir -p .wrap & node wrap.js
	./node_modules/.bin/node-pre-gyp build

# Create package
package:
	./node_modules/.bin/node-pre-gyp package

# Publish package
# Unpublish first because node-pre-gyp does not allow "publish over existing version"
publish: package
ifeq (dirty,$(DIRTY))
	$(error Publish not done for dirty versions. Commit or stash and try again.)
else
	export node_pre_gyp_accessKeyId="$$AWS_ACCESS_KEY" ;\
	export node_pre_gyp_secretAccessKey="$$AWS_SECRET_KEY" ;\
	  ./node_modules/.bin/node-pre-gyp unpublish ;\
	  ./node_modules/.bin/node-pre-gyp publish
	$(call PUBLISH_NOTIFY,node,$(NODE_VERSION),$(OS)/$(ARCH))
endif

# Run tests
test:
	npm test

# Run tests with coverage
cover:
	npm run cover

# Install package globally
install:
	sudo npm install -g

# Clean up
clean:
	rm -rf build lib coverage .wrap

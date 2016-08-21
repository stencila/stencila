include ../shared.make

all: setup build

.PHONY: build

setup:
	npm install

build:
	./node_modules/.bin/gulp build

watch:
	./node_modules/.bin/gulp watch

examples:
	stencila-r examples/a render write page:"index.html"
	stencila-r examples/stencil-with-pars render write page:"index.html"
	stencila-r examples/b update write page:"index.html"
	stencila-r examples/sheet-with-error update write page:"index.html"

serve:
	node server.js

serve-hub:
	node server.js --upstream=https://stenci.la

# Run tests
test:
	npm test

# Run tests in node with debugger
test-debug:
	node-debug  tests

# Run tests with coverage
cover:
	npm run cover

publish:
ifeq (dirty,$(DIRTY))
	$(error Publish not done for dirty versions. Commit or stash and try again.)
else
	aws s3 sync build s3://get.stenci.la/web/
	$(call PUBLISH_NOTIFY,web,ES5)
endif

clean:
	rm -rf build

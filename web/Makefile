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

devserve:
	node server.js

devserve-hub:
	node server.js https://stenci.la

publish:
ifeq (dirty,$(DIRTY))
	$(error Publish not done for dirty versions. Commit or stash and try again.)
else
	aws s3 sync build s3://get.stenci.la/web/
	$(call PUBLISH_NOTIFY,web,ES5)
endif

clean:
	rm -rf build

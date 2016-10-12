all: setup build

.PHONY: build

setup:
	npm install

build:
	npm run build

watch:
	npm run watch

run:
	npm start

run-hub:
	node server.js --upstream=https://stenci.la

lint:
	npm run lint

test:
	npm test

test-unit:
	npm run test-unit

test-fun:
	npm run test-fun

test-one/%:
	node tests/one $*

test-debug:
	node-debug tests

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

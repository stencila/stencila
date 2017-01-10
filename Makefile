all: setup lint cover docs

setup:
	npm install

lint:
	npm run lint

test:
	npm test

test-browser:
	npm run test-browser

cover:
	npm run cover

docs:
	npm run docs
.PHONY: docs

docs-serve:
	npm run docs-serve

clean:
	rm -rf node_modules

all: setup lint test build docs

setup:
	npm install

hooks:
	cp pre-commit.sh .git/hooks/pre-commit

lint:
	npm run lint

test:
	npm test

cover:
	npm run cover

run:
	npm start

build:
	npm run build
.PHONY: build

docs:
	npm run docs
.PHONY: docs

clean:
	npm run clean

all: setup check test build docs

setup:
	npm install

lint:
	npm run lint

check:
	npm run check

test:
	npm test

build:
	npm run build

watch:
	npm run watch

docs:
	npm run docs

clean:
	npm run clean

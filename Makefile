all: setup check test build docs

setup:
	npm install

hooks:
	cp pre-commit.sh .git/hooks/pre-commit

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

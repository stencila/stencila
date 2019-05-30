all: setup lint cover build

setup:
	npm install

lint:
	npm run lint

test:
	npm test

cover:
	npm run cover

build:
	npm run build

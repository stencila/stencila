all: format lint test build

node_modules: package.json
	npm install

setup: node_modules

format: setup
	npm run format

lint: setup
	npm run lint:fix

test: setup
	npm run test

cover: setup
	npm run cover

build: setup
	npm run build

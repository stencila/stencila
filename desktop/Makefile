node_modules: package.json
	npm install

setup: node_modules

lint: setup
	npm run lint

test: setup
	npm run test -- --ci

test-e2e:
	npm run test:e2e

build: setup
	npm run ui:build
	npm run make

publish: setup
	npm run ui:build
	npm run publish

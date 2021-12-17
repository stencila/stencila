node_modules: package.json
	npm install

setup: node_modules

lint: setup
	npm run lint

test: setup
	npm run test -- --ci

build: setup
	npm run ui:build
	npm run make

publish: setup
	npm run ui:build
	npm run publish

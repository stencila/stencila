node_modules: package.json
	npm install

setup: node_modules

lint: setup
	npm run lint

test: setup
	npm run test

build: setup
	npm run ui:build
	npm run make

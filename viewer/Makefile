node_modules: package.json
	npm install

setup: node_modules

run: setup
	npm start

# Removes PWA related files from the top level of the build
# folder to avoid embedding these in binaries uncessarily
build: setup
	npm run build
	rm build/*.json build/*.js
.PHONY: build

test: setup
	npm test

node_modules: package.json
	npm install

setup: node_modules

run: setup
	npm start

# Needs to move build/.gitignore to avoid it getting deleted.
# Removes PWA related files from the top level of the build
# folder to avoid embedding these in binaries uncessarily
build: setup
	mv build/.gitignore .build-gitignore
	npm run build
	mv .build-gitignore build/.gitignore
	rm build/*.json build/*.js
.PHONY: build

test: setup
	npm test

all: clean demos references build

node_modules: package.json
	npm install

setup: node_modules

# Start the development server
# Generates fake CLI demo .cast files if necessary so
# does not fail if these have not yet been created
run:
	make -C demos/cli touch
	npm start

# Build demos folder
demos:
	make -C demos/cli
.PHONY: demos

# Build references folder
references:
	make -C references/schema
.PHONY: references

# Bundle site into static files for production
build:
	npm run build
.PHONY: build

# Publish site to GitHub pages
deploy:
	npm run deploy

# Remove cached / fetched files
clean:
	npm run clear
	make -C demos/cli clean
	make -C references/schema clean

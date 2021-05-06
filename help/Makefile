all: clean demos references build

# Start the development server
run:
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

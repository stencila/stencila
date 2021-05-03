all: clean demos reference build

# Start the development server
run:
	npm start

# Build demos folder
demos:
	make -C docs/demos/cli

# Build reference folder
reference:
	make -C docs/reference/schema

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
	make -C docs/demos/cli clean
	make -C docs/reference/schema clean

# Start the development server
run:
	npm start

# Build demos
demos:
	make -C docs/demos/cli

# Build resources
resources:
	make -C docs/reference/schema

# Bundle site into static files for production
build:
	npm run build
.PHONY: build

# Publish site to GitHub pages
deploy:
	npm run deploy

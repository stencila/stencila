all: fix test audit build

# Install dependencies
install:
ifdef CI
	cd .. && npm ci --ignore-scripts
else
	cd .. && npm install --ignore-scripts
endif
	make -C ../ts build

# Make formatting and linting fixes
fix: install
	npm run fix

# Run linting checks
lint: install
	npm run lint

# Run tests
test: install
	npm run test

# Run watch
run: install
	npm start

# List outdated dependencies
outdated: install
	npm outdated

# Audit dependencies
audit: install
	npm run unused
	cd .. && npm run audit

# Build the package
build: install clean
	npm run build

# Build in docker
.docker:
	docker build --no-cache -t stencila/web:latest .
	touch $@
build-in-docker: .docker
	docker run --rm -it \
		-v ./dist:/home/node/app/dist \
		stencila/web:latest \
		bash -c "npm run build"

# Clean up development artifacts
clean:
	rm -f .docker
	mv dist/.gitignore dist.gitignore
	rm -rf dist
	mkdir dist
	mv dist.gitignore dist/.gitignore

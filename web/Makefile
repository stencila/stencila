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
	npm run typecheck

# Run tests
test: install
	npm run test

# Install playwright with chromium
playwright: install
	npx playwright install --with-deps chromium

# Run end-to-end tests
e2e: playwright build
	cargo build -p cli
	npm run e2e

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

# Build the distribution
build: install clean
	npm run build

# Publish the distribution (requires rclone and relevant secrets)
publish: build
	RCLONE_CONFIG_R2_TYPE=s3 \
	RCLONE_CONFIG_R2_PROVIDER=Cloudflare \
	RCLONE_CONFIG_R2_ENV_AUTH=false \
	RCLONE_CONFIG_R2_ENDPOINT=https://$(CF_ACCOUNT_ID).r2.cloudflarestorage.com \
	RCLONE_CONFIG_R2_ACCESS_KEY_ID=$(R2_ACCESS_KEY_ID) \
	RCLONE_CONFIG_R2_SECRET_ACCESS_KEY=$(R2_SECRET_ACCESS_KEY) \
	rclone copy dist R2:web-dist/$(VERSION)  --immutable --checksum --progress

# Clean up development artifacts
clean:
	mv dist/.gitignore dist.gitignore
	rm -rf dist
	mkdir dist
	mv dist.gitignore dist/.gitignore

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
	cargo build -p stencila-cli
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
# If there is no VERSION or BASE_URL, vite.config.ts defaults to /~static/dev/
build: install
	@if [ -n "$(BASE_URL)" ]; then \
		BASE_URL=$(BASE_URL) npm run build; \
	elif [ "$(VERSION)" = "dev" ]; then \
		BASE_URL=https://stencila.dev/web/dev npm run build; \
	elif [ -n "$(VERSION)" ]; then \
		BASE_URL=https://stencila.dev/web/v$(VERSION) npm run build; \
	else \
		npm run build; \
	fi

# Publish the distribution (requires rclone and relevant secrets)
# When VERSION=dev, use rclone sync (mutable, overwrites existing files).
# For tagged releases, use rclone copy --immutable (fails if files already exist).
publish: build
ifeq ($(VERSION),dev)
	RCLONE_CONFIG_R2_TYPE=s3 \
	RCLONE_CONFIG_R2_PROVIDER=Cloudflare \
	RCLONE_CONFIG_R2_ENV_AUTH=false \
	RCLONE_CONFIG_R2_ENDPOINT=https://$(CF_ACCOUNT_ID).r2.cloudflarestorage.com \
	RCLONE_CONFIG_R2_ACCESS_KEY_ID=$(R2_ACCESS_KEY_ID) \
	RCLONE_CONFIG_R2_SECRET_ACCESS_KEY=$(R2_SECRET_ACCESS_KEY) \
	rclone sync dist R2:web-dist/dev --checksum --progress
else
	RCLONE_CONFIG_R2_TYPE=s3 \
	RCLONE_CONFIG_R2_PROVIDER=Cloudflare \
	RCLONE_CONFIG_R2_ENV_AUTH=false \
	RCLONE_CONFIG_R2_ENDPOINT=https://$(CF_ACCOUNT_ID).r2.cloudflarestorage.com \
	RCLONE_CONFIG_R2_ACCESS_KEY_ID=$(R2_ACCESS_KEY_ID) \
	RCLONE_CONFIG_R2_SECRET_ACCESS_KEY=$(R2_SECRET_ACCESS_KEY) \
	rclone copy dist R2:web-dist/$(VERSION) --immutable --checksum --progress
endif

# Clean up development artifacts
clean:
	rm -rf dist
	mkdir dist
	@printf '# Ignore everything in this folder (except this file)\n# to ensure that the folder exists in the repo so that it\n# is available for embedding in the Rust CLI\n*\n!.gitignore\n' > dist/.gitignore

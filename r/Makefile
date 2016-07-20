include ../shared.make

all: setup build test
.PHONY: setup build check test cover publish install clean

# R-specific version format
VERSION := $(shell ../version.sh r)

# Install requirements for build
setup:
	Rscript -e "install.packages(c('Rcpp','codetools','roxygen2','RUnit','covr'),repo='http://cloud.r-project.org/')"

# Build package
build:
	cp DESCRIPTION.txt stencila/DESCRIPTION
	sed -i 's!Version:.*$$!Version: $(VERSION)!' stencila/DESCRIPTION
	sed -i 's!Date:.*$$!Date: $(shell date -u +%Y-%m-%dT%H:%M:%SZ)!' stencila/DESCRIPTION
	Rscript -e "roxygen2::roxygenize('stencila');"
ifeq ($(OS),win)
	R CMD INSTALL --build --no-multiarch stencila
else
	R CMD build stencila
endif

# Check package
# Requires pfdlatex to be installed
check:
	R CMD check stencila

# Run tests (does a local install first)
test:
	R CMD INSTALL stencila
	cd stencila/tests && Rscript test-runit.R
	cd stencila/tests && Rscript test-spreadsheet.R

# Run tests with coverage
cover:
	Rscript -e 'cov<-covr::package_coverage("stencila"); cat(covr:::to_codecov(cov),file="coverage.json"); print(cov)'

# Publish package into a local R repository for mirroring to http://get.stenci.la/r
ifeq ($(OS),win)
	PACKAGE_EXT = zip
	DLL_EXT = dll
	REPO_DIR = bin/windows/contrib/$(VERSION)
	REPO_TYPE = win.binary
else
	PACKAGE_EXT = tar.gz
	DLL_EXT = so
	REPO_DIR = src/contrib
	REPO_TYPE = source
endif
PACKAGE = stencila_$(VERSION).$(PACKAGE_EXT)
publish:
ifeq (dirty,$(DIRTY))
	$(error Publish is not done for dirty versions. Commit or stash and try again.)
else
	mkdir -p build/repo/$(REPO_DIR)
	cp $(PACKAGE) build/repo/$(REPO_DIR)
	Rscript -e "tools::write_PACKAGES('build/repo/$(REPO_DIR)',type='$(REPO_TYPE)')"

	aws s3 cp --recursive build/repo/$(REPO_DIR) s3://get.stenci.la/r/$(REPO_DIR)
	$(call PUBLISH_NOTIFY,r,$(R_VERSION),$(OS)/$(ARCH),http://get.stenci.la/r/$(REPO_DIR)/$(PACKAGE))
endif

# Install package
install:
	R CMD INSTALL stencila
	Rscript -e 'stencila:::install()'

# Clean up after build
clean:
	rm -rf stencila/DESCRIPTION stencila/NAMESPACE stencila/man stencila/src/*.{o,dll,so} stencila/inst/libs \
		stencila.Rcheck stencila_*.tar.gz stencila_*.zip


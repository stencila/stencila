include ../shared.make

.PHONY: build

VERSION := $(shell ../version.sh r)
R_VERSION := $(shell Rscript -e "cat(R.version\$$major,strsplit(R.version\$$minor,'.',fixed=T)[[1]][1],sep='.')" )

ifeq ($(OS),win)
	PACKAGE_EXT = zip
	DLL_EXT = dll
	REPO_DIR = bin/windows/contrib/$(VERSION)
	REPO_TYPE = win.binary
	# Extra DLLs needed on windows. These should be available from the MSYS2 install.
	# List of extra DLLs required can be determined by running Dependency Walker
	# (http://www.dependencywalker.com/) on stencila.dll
	DLL_EXTRAS = $(patsubst %, /c/msys64/mingw64/bin/%, \
		libeay32.dll libgcc_s_seh-1.dll libstdc++-6.dll libwinpthread-1.dll ssleay32.dll zlib1.dll)
else
	PACKAGE_EXT = tar.gz
	DLL_EXT = so
	REPO_DIR = src/contrib
	REPO_TYPE = source
endif

# Filenames of dll, zipped dll and package
DLL = stencila.$(DLL_EXT)
ZIP = stencila-$(VERSION)-r-$(R_VERSION)-$(OS)-$(ARCH).zip
PACKAGE = stencila_$(VERSION).$(PACKAGE_EXT)

CPP_FLAGS := $(shell R CMD config --cppflags) $(shell Rscript -e "Rcpp:::CxxFlags()")
LD_FLAGS := $(shell R CMD config --ldflags) $(shell Rscript -e "Rcpp:::LdFlags()")

# List of object files to be compiled
OBJECTS = $(patsubst extension/%.cpp,build/%.o,$(wildcard extension/*.cpp))

# Install requirements for build
setup:
	sudo Rscript -e "install.packages(c('Rcpp','codetools','roxygen2','RUnit','covr'),repo='http://cloud.r-project.org/')"

# Compile an extension file
build/%.o: extension/%.cpp
	@mkdir -p build
	$(CXX) $(CPP_FLAGS) -fPIC -std=c++11 -I../cpp \
	   -I../cpp/requires/boost \
	   -I../cpp/requires/websocketpp \
	   -I../cpp/requires/cpp-netlib -o$@ -c $<
	
# Create dll and check it can be loaded
build/$(DLL): $(OBJECTS)
	$(CXX) -shared -o$@ $^ $(LD_FLAGS) -L../cpp/library -lstencila -lssl -lcurl
	Rscript -e "dyn.load('build/$(DLL)')"

# Create zipped dll
build/$(ZIP): build/$(DLL)
	zip -j $@ $< $(DLL_EXTRAS)

# Build package
build: build/$(ZIP)
	cp DESCRIPTION.txt stencila/DESCRIPTION
	sed -i 's!Version:.*$$!Version: $(VERSION)!' stencila/DESCRIPTION
	sed -i 's!Date:.*$$!Date: $(shell date -u +%Y-%m-%dT%H:%M:%SZ)!' stencila/DESCRIPTION
	Rscript -e "roxygen2::roxygenize('stencila');"
	STENCILA_R_BUILD=$(realpath build) R CMD check stencila
ifeq ($(OS),win)
	R CMD INSTALL --build stencila
else
	R CMD build stencila
endif

# Build package complete with zipped DLL
# Used for easier installation into Docker images
build-complete:
	cp -f build/$(ZIP) stencila/inst/bin
	tar -czf stencila_$(VERSION)_complete.tar.gz stencila
	rm stencila/inst/bin/*.zip

# Test the package
test:
	STENCILA_R_BUILD=$(realpath build) R CMD check stencila

# Run tests with coverage
cover:
	Rscript -e 'cov<-covr::package_coverage("stencila"); cat(covr:::to_codecov(cov),file="coverage.json"); print(cov)'

# Publish package into a local R repository for mirroring to http://get.stenci.la/r
publish:
ifeq (dirty,$(DIRTY))
	$(error Publish is not done for dirty versions. Commit or stash and try again.)
else
	mkdir -p build/repo/$(REPO_DIR)
	cp $(PACKAGE) build/repo/$(REPO_DIR)
	Rscript -e "tools::write_PACKAGES('build/repo/$(REPO_DIR)',type='$(REPO_TYPE)')"

	aws s3 cp build/$(ZIP) s3://get.stenci.la/r/dll/
	aws s3 cp --recursive build/repo/$(REPO_DIR) s3://get.stenci.la/r/$(REPO_DIR)
	$(call PUBLISH_NOTIFY,r,$(R_VERSION),$(OS)/$(ARCH),http://get.stenci.la/$(REPO_DIR)/$(PACKAGE))
endif

# Install package
install:
	STENCILA_R_BUILD=$(realpath build) R CMD INSTALL stencila
	sudo Rscript -e 'stencila::install()'

# Clean up after build
clean:
	rm -rf build stencila/NAMESPACE stencila/man stencila.Rcheck stencila_*.tar.gz

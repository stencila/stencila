include ../shared.make

.PHONY: build

VERSION := $(shell ../version.sh r)

R_VERSION := $(shell Rscript -e "cat(R.version\$$major,strsplit(R.version\$$minor,'.',fixed=T)[[1]][1],sep='.')" )

ifeq ($(OS),win)
	PACKAGE_EXT := zip
	DLL_EXT := dll
	REPO_DIR := bin/windows/contrib/$(VERSION)
	REPO_TYPE := win.binary
	# Extra DLLs needed on windows. These should be available from the MSYS2 install.
	# List of extra DLLs required can be determined by running Dependency Walker
	# (http://www.dependencywalker.com/) on stencila.dll
	DLL_EXTRAS := $(patsubst %, /c/msys64/mingw64/bin/%, \
		libeay32.dll libgcc_s_seh-1.dll libstdc++-6.dll libwinpthread-1.dll ssleay32.dll zlib1.dll)
else
	PACKAGE_EXT := tar.gz
	DLL_EXT := so
	REPO_DIR := src/contrib
	REPO_TYPE := source
endif

CPP_FLAGS := $(shell R CMD config --cppflags) $(shell Rscript -e "Rcpp:::CxxFlags()")
LD_FLAGS := $(shell R CMD config --ldflags) $(shell Rscript -e "Rcpp:::LdFlags()")

# List of object files to be compiled
OBJECTS = $(patsubst extension/%.cpp,build/%.o,$(wildcard extension/*.cpp))

# Filenames of dll and zipped dll
DLL := stencila.$(DLL_EXT)
ZIP := stencila-$(VERSION)-r-$(R_VERSION)-$(OS)-$(ARCH).zip

# Compile an exension file
build/%.o: extension/%.cpp
	@mkdir -p build
	$(CXX) $(CPP_FLAGS) -fPIC -std=c++11 -I../cpp \
	   -I../build/current/cpp/requires/boost \
	   -I../build/current/cpp/requires/websocketpp \
	   -I../build/current/cpp/requires/cpp-netlib -o$@ -c $<
	
# Create dll and check it can be loaded
build/$(DLL): $(OBJECTS)
	$(CXX) -shared -o$@ $^ $(LD_FLAGS) -L../build/current/cpp/library -lstencila -lssl -lcurl
	Rscript -e "dyn.load('build/$(DLL)')"

# Create zipped dll
build/$(ZIP): build/$(DLL)
	zip -j $@ $< $(DLL_EXTRAS)

# Build package
build: build/$(ZIP)
	sed -i 's!Version: .*$$!Version: $(VERSION)!' stencila/DESCRIPTION
	sed -i 's!Date: .*$$!Date: $(shell date -u +%Y-%m-%dT%H:%M:%SZ)!' stencila/DESCRIPTION
	Rscript -e "library(roxygen2); roxygenize('stencila');"
	STENCILA_R_BUILD=$(realpath build) R CMD check stencila
ifeq ($(OS),win)
	R CMD INSTALL --build stencila
else
	R CMD build stencila
endif

install:
	R CMD INSTALL stencila
	sudo Rscript -e 'library(stencila); stencila:::install()'

clean:
	rm -rf build stencila/NAMESPACE stencila/man stencila.Rcheck


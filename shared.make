# Get the operating system  e.g. linux
OS := $(shell uname -s | tr A-Z a-z)
ifeq ($(OS),darwin)
    OS := osx
endif
ifeq ($(findstring msys,$(OS)),msys)
	OS := win
endif
ifeq ($(findstring mingw,$(OS)),mingw)
	OS := win
endif

# Get the machine architecture e.g i386, x86_64
ARCH = $(shell uname -m)

# Stencila version and commit
VERSION = $(shell $(dir $(lastword $(MAKEFILE_LIST)))version.sh)
COMMIT = $(shell git rev-parse HEAD)

# Check if the repository is dirty i.e. has uncommitted changes
DIRTY = $(findstring dirty,$(shell git describe --dirty))

# Show defined variables
vars:
	@echo OS : $(OS)
	@echo ARCH : $(ARCH)
	@echo VERSION : $(VERSION)
	@echo COMMIT : $(COMMIT)
	@echo DIRTY : $(DIRTY)

# Notify the Stencila hub that a build has been published
define PUBLISH_NOTIFY
	curl -u "Token:$$STENCILA_TOKEN" \
	  -X POST -H "Content-Type: application/json" -H "Accept: application/json" -d "{ \
	    \"package\": \"$1\", \
	    \"flavour\": \"$2\", \
	    \"platform\": \"$3\", \
	    \"url\": \"$4\", \
	    \"version\": \"$(VERSION)\", \
	    \"commit\": \"$(COMMIT)\" \
	   }" "https://stenci.la/builds"
endef

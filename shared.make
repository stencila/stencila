# Get the operating system  e.g. linux
OS := $(shell uname -s | tr A-Z a-z)
ifeq ($(OS),darwin)
    OS := 'osx'
endif

# Get the machine architecture e.g i386, x86_64
ARCH := $(shell uname -m)

# Check if the repository is dirty i.e. has uncommitted changes
DIRTY = $(findstring dirty,$(shell git describe --dirty))

# Show defined variables
vars:
	@echo OS : $(OS)
	@echo ARCH : $(ARCH)
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

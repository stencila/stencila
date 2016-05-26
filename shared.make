# Get the operating system  e.g. linux
OS := $(shell uname -s | tr A-Z a-z)
ifeq ($(OS),darwin)
    OS := 'osx'
endif

# Get the machine architecture e.g i386, x86_64
ARCH := $(shell uname -m)

# Show defined variables
vars:
	@echo OS: $(OS)
	@echo ARCH: $(ARCH)

# Assert that the repository is not dirty i.e. no uncommitted changes
define ASSERT_CLEAN
	ifeq ($(findstring dirty,$(shell git describe --dirty)),dirty)
	  $(error Uncommitted changes. Commit or stash and then try again.)
	endif
endef

# Notify the Stencila hub that a build has been delivered
define DELIVER_NOTIFY
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

include ../shared.make

all: setup build cover

.PHONY: setup build test cover install clean

# Get Python version major.minor
PY_VERSION := $(shell python -c "import sys; sys.stdout.write('%s.%s' % (sys.version_info[0], sys.version_info[1]))")

# Get the filename of the latest wheel
WHEEL_FILE = $(shell cd dist ; ls -rt *.whl | tail -n 1)

# For access to tox when it is installed under the user scheme
PATH := ~/.local/bin:$(PATH)

# Install build requirements
setup:
	pip install --user setuptools wheel tox

# Build package 
build:
	python setup.py bdist_wheel

# Run tests
test:
ifeq ($(OS), win)
# Virtalenv fails to install on MSYS2 (see issue #200)
# So this workaround installs the stancila package and runs tests "manually" (i.e. not using tox/virtualenv)
	pip install --user --upgrade --force-reinstall dist/$(WHEEL_FILE)
	python tests/test_stencila.py
else
	export PATH=$(PATH) ; tox --installpkg dist/$(WHEEL_FILE)
endif

# Run coverage
cover:
	export PATH=$(PATH) ; tox -e cover --installpkg dist/$(WHEEL_FILE)
	sed -i "s!.tox/cover/lib/python2.7/site-packages/stencila/!py/stencila/!g" coverage.xml

# Install package
install:
	pip install --user --upgrade --force-reinstall dist/$(WHEEL_FILE)

# Publish package
publish:
ifeq (dirty,$(DIRTY))
	$(error Publish not done for dirty versions. Commit or stash and try again.)
else
	aws s3 cp dist/$(WHEEL_FILE) s3://get.stenci.la/py/
	$(call PUBLISH_NOTIFY,py,$(PY_VERSION),$(OS)/$(ARCH),http://get.stenci.la/py/$(WHEEL_FILE))
endif

# Clean up
clean:
	rm -rf build dist stencila.egg-info .coverage coverage.xml

include ../shared.make

all: setup build cover

.PHONY: setup build test cover install clean

# Get Python version major.minor
PY_VERSION := $(shell python -c "import sys; sys.stdout.write('%s.%s' % (sys.version_info[0], sys.version_info[1]))")

# Get the filename of the latest wheel
WHEEL_FILE = $(shell cd dist ; ls -rt *.whl | tail -n 1)

# Install build requirements
setup:
	pip install setuptools wheel tox

# Build package
build:
	python setup.py bdist_wheel

# Run tests
test:
	tox --installpkg dist/$(WHEEL_FILE)

# Run coverage
cover:
	tox -e cover --installpkg dist/$(WHEEL_FILE)
	sed -i "s!.tox/cover/lib/python2.7/site-packages/stencila/!py/stencila/!g" coverage.xml

# Install package
install:
	pip install --upgrade --force-reinstall dist/$(WHEEL_FILE)

# Deliver package
deliver:
	$(call ASSERT_CLEAN)
	#aws s3 cp dist/$(WHEEL_FILE) s3://get.stenci.la/py/
	#$(call DELIVER_NOTIFY,py,$(PY_VERSION),$(OS)/$(ARCH),http://get.stenci.la/py/$(WHEEL_FILE))

# Clean up
clean:
	rm -rf build dist stencila.egg-info .coverage coverage.xml

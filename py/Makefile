all: regen lint format test build docs

setup:
	pip3 install -r requirements-dev.txt

install:
	python3 setup.py install

regen:
	npm run build:py

lint:
	pylint stencila/schema
	mypy stencila/schema --ignore-missing-imports

format:
	black --exclude stencila/schema/types.py ./stencila ./*.py

test:
	tox

build:
	python3 setup.py sdist bdist_wheel
.PHONY: build

docs:
	pdoc --html --output-dir docs --force ./stencila/schema
	mv docs/schema/* docs
	rmdir docs/schema
.PHONY: docs

clean:
	rm -rf build dist .mypy_cache .coverage coverage.xml htmlcov .tox *.egg-info **/__pycache__
